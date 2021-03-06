#![allow(clippy::collapsible_else_if)]

extern crate ic_cdk_macros;
extern crate serde;
extern crate ic_cron;

use std::borrow::{Cow};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::mem;
use std::num::TryFromIntError;
use std::result::Result as StdResult;

//use candid::{CandidType, Encode, Principal};
use ic_cdk::{api::{self, call}, storage};
use ic_certified_map::Hash;
use include_base64::include_base64;

use std::collections::BTreeMap;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use ed25519_dalek::{PublicKey, Signature, SignatureError, Verifier};
// use ic_cdk::export::Principal;
use ic_cdk_macros::*;

use ic_cdk::{
    export::{
        candid::CandidType,
        Principal,
    },
};
use ic_cron::types::Iterations;
//use serde::__private::de::Content::String;
use serde::Deserialize;

mod http;

ic_cron::implement_cron!();

const MGMT: Principal = Principal::from_slice(&[]);

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[derive(CandidType, Deserialize)]
enum TaskKind {
    CheckNfts,
}

// enqueue a task
#[ic_cdk_macros::update]
fn cron_check_data_nfts() {
    ic_cdk::print("Start cron_check_data_nfts");
    cron_enqueue(
        // set a task payload - any CandidType is supported
        TaskKind::CheckNfts,
        // set a scheduling interval (how often and how many times to execute)
        ic_cron::types::SchedulingOptions {
            delay_nano: 1_000_000_000 * 60 * 2, // start after 2 minutes
            interval_nano: 1_000_000_000 * 60 * 60 * 24, // each day
            iterations: Iterations::Infinite, // infinite
        },
    );
}

#[heartbeat]
fn bumbum() {
    for task in cron_ready_tasks() {
        let kind = task
            .get_payload::<TaskKind>()
            .expect("Unable to deserialize cron task kind");

        match kind {
            TaskKind::CheckNfts => {
                let nfts = STATE.with(|state| {
                    let mut state = state.borrow();
                    state.nfts.clone()
                });

                ic_cdk::print("Start checking date of nfts...");
                for token_id in 0..nfts.len() {
                    is_date_expired_nft(token_id as u64);
                }
                ic_cdk::print("--> terminate checking date of nfts <--");
            }
        }
    }
}

#[derive(CandidType, Deserialize)]
struct StableState {
    state: State,
    hashes: Vec<(String, Hash)>,
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|state| mem::take(&mut *state.borrow_mut()));
    let hashes = http::HASHES.with(|hashes| mem::take(&mut *hashes.borrow_mut()));
    let hashes = hashes.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let stable_state = StableState { state, hashes };
    storage::stable_save((stable_state,)).unwrap();
}

/*
#[post_upgrade]
fn post_upgrade() {
    let (StableState { state, hashes },) = storage::stable_restore().unwrap();
    STATE.with(|state0| *state0.borrow_mut() = state);
    let hashes = hashes.into_iter().collect();
    http::HASHES.with(|hashes0| *hashes0.borrow_mut() = hashes);
}
*/

#[derive(CandidType, Deserialize)]
struct InitArgs {
    custodians: Option<HashSet<Principal>>,
    logo: Option<LogoResult>,
    name: String,
    symbol: String,
}


#[init]
fn init() {
    cron_check_data_nfts();
    /*
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.custodians = args
            .custodians
            .unwrap_or_else(|| HashSet::from_iter([api::caller()]));
        state.name = args.name;
        state.symbol = args.symbol;
        state.logo = args.logo;
    });
     */
}

#[derive(CandidType, Deserialize)]
enum Error {
    Unauthorized,
    InvalidTokenId,
    ZeroAddress,
    Other,
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Self::InvalidTokenId
    }
}

type Result<T = u128, E = Error> = StdResult<T, E>;

// --------------
// base interface
// --------------

#[query(name = "balanceOfDip721")]
fn balance_of(user: Principal) -> u64 {
    STATE.with(|state| {
        state
            .borrow()
            .nfts
            .iter()
            .filter(|n| n.owner == user)
            .count() as u64
    })
}

#[query(name = "ownerOfDip721")]
fn owner_of(token_id: u64) -> Result<Principal> {
    STATE.with(|state| {
        let owner = state
            .borrow()
            .nfts
            .get(usize::try_from(token_id)?)
            .ok_or(Error::InvalidTokenId)?
            .owner;
        Ok(owner)
    })
}

#[update(name = "transferFromDip721")]
fn transfer_from(from: Principal, to: Principal, token_id: u64) -> Result {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let state = &mut *state;
        let nft = state
            .nfts
            .get_mut(usize::try_from(token_id)?)
            .ok_or(Error::InvalidTokenId)?;
        let caller = api::caller();
        if nft.owner != caller
            && nft.approved != Some(caller)
            && !state
            .operators
            .get(&from)
            .map(|s| s.contains(&caller))
            .unwrap_or(false)
            && !state.custodians.contains(&caller)
        {
            Err(Error::Unauthorized)
        } else if nft.owner != from {
            Err(Error::Other)
        } else {
            nft.approved = None;
            nft.owner = to;
            Ok(state.next_txid())
        }
    })
}

#[update(name = "safeTransferFromDip721")]
fn safe_transfer_from(from: Principal, to: Principal, token_id: u64) -> Result {
    if to == MGMT {
        Err(Error::ZeroAddress)
    } else {
        transfer_from(from, to, token_id)
    }
}

#[query(name = "supportedInterfacesDip721")]
fn supported_interfaces() -> &'static [InterfaceId] {
    &[
        InterfaceId::TransferNotification,
        // InterfaceId::Approval, // Psychedelic/DIP721#5
        InterfaceId::Burn,
        InterfaceId::Mint,
    ]
}

#[derive(CandidType, Deserialize, Clone)]
struct LogoResult {
    logo_type: Cow<'static, str>,
    data: Cow<'static, str>,
}

#[export_name = "canister_query logoDip721"]
fn logo() /* -> &'static LogoResult */
{
    ic_cdk::setup();
    STATE.with(|state| call::reply((state.borrow().logo.as_ref().unwrap_or(&DEFAULT_LOGO),)))
}


#[query(name = "symbolDip721")]
fn symbol() -> String {
    STATE.with(|state| state.borrow().symbol.clone())
}

const DEFAULT_LOGO: LogoResult = LogoResult {
    data: Cow::Borrowed(include_base64!("logo.png")),
    logo_type: Cow::Borrowed("image/png"),
};

#[query(name = "totalSupplyDip721")]
fn total_supply() -> u64 {
    STATE.with(|state| state.borrow().nfts.len() as u64)
}

#[export_name = "canister_query getMetadataDip721"]
fn get_metadata(/* token_id: u64 */) /* -> Result<&'static MetadataDesc> */
{
    ic_cdk::setup();
    let token_id = call::arg_data::<(u64,)>().0;
    let res: Result<()> = STATE.with(|state| {
        let state = state.borrow();
        let metadata = &state
            .nfts
            .get(usize::try_from(token_id)?)
            .ok_or(Error::InvalidTokenId)?
            .metadata;
        call::reply((Ok::<_, Error>(metadata),));
        Ok(())
    });
    if let Err(e) = res {
        call::reply((Err::<MetadataDesc, _>(e),));
    }
}

#[derive(CandidType)]
struct ExtendedMetadataResult<'a> {
    metadata_desc: MetadataDescRef<'a>,
    token_id: u64,
}

#[export_name = "canister_update getMetadataForUserDip721"]
fn get_metadata_for_user(/* user: Principal */) /* -> Vec<ExtendedMetadataResult> */
{
    ic_cdk::setup();
    let user = call::arg_data::<(Principal,)>().0;
    STATE.with(|state| {
        let state = state.borrow();
        let metadata: Vec<_> = state
            .nfts
            .iter()
            .filter(|n| n.owner == user)
            .map(|n| ExtendedMetadataResult {
                metadata_desc: &n.metadata,
                token_id: n.id,
            })
            .collect();
        call::reply((metadata,));
    });
}

// ------------------
// approval interface
// ------------------

#[update(name = "approveDip721")]
fn approve(user: Principal, token_id: u64) -> Result {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let state = &mut *state;
        let caller = api::caller();
        let nft = state
            .nfts
            .get_mut(usize::try_from(token_id)?)
            .ok_or(Error::InvalidTokenId)?;
        if nft.owner != caller
            && nft.approved != Some(caller)
            && !state
            .operators
            .get(&user)
            .map(|s| s.contains(&caller))
            .unwrap_or(false)
            && !state.custodians.contains(&caller)
        {
            Err(Error::Unauthorized)
        } else {
            nft.approved = Some(user);
            Ok(state.next_txid())
        }
    })
}

#[update(name = "setApprovalForAllDip721")]
fn set_approval_for_all(operator: Principal, is_approved: bool) -> Result {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let caller = api::caller();
        if operator != caller {
            let operators = state.operators.entry(caller).or_default();
            if operator == MGMT {
                if !is_approved {
                    operators.clear();
                } else {
                    // cannot enable everyone as an operator
                }
            } else {
                if is_approved {
                    operators.insert(operator);
                } else {
                    operators.remove(&operator);
                }
            }
        }
        Ok(state.next_txid())
    })
}

// #[query(name = "getApprovedDip721")] // Psychedelic/DIP721#5
fn _get_approved(token_id: u64) -> Result<Principal> {
    STATE.with(|state| {
        let approved = state
            .borrow()
            .nfts
            .get(usize::try_from(token_id)?)
            .ok_or(Error::InvalidTokenId)?
            .approved
            .unwrap_or_else(api::caller);
        Ok(approved)
    })
}

#[query(name = "isApprovedForAllDip721")]
fn is_approved_for_all(operator: Principal) -> bool {
    STATE.with(|state| {
        state
            .borrow()
            .operators
            .get(&api::caller())
            .map(|s| s.contains(&operator))
            .unwrap_or(false)
    })
}

// --------------
// mint interface
// --------------


#[update(name = "mintDip721")]
fn mint(
    name: String,
    to: Principal,
    metadata: MetadataDesc
    //blob_content: Vec<u8>
) -> Result<MintResult, ConstrainedError> {
    let (txid, tkid) = STATE.with(|state| {
        let mut state = state.borrow_mut();
        /*if !state.custodians.contains(&api::caller()) {
            return Err(ConstrainedError::Unauthorized);
        }*/

        let new_id = state.nfts.len() as u64;
        let nft = Nft {
            owner: to,
            approved: None,
            id: new_id,
            metadata,
            //content: blob_content,
            name: name,
        };
        state.nfts.push(nft);
        Ok((state.next_txid(), new_id))
    })?;
    http::add_hash(tkid);
    Ok(MintResult {
        id: txid,
        token_id: tkid,
    })
}

#[query(name = "nameDip721")]
fn name(token_id:u64) -> String {
    let id_input = usize::try_from(token_id);
    if id_input.is_err() {
        return String::from("Errore input");
    }
    let token_id = id_input.unwrap();

    let nft = STATE.with(|state| {
        let mut state = state.borrow_mut();
        state
            .nfts
            .get(token_id).cloned()
    });
    return if nft.is_some() {
        String::from(nft.unwrap().name)
    } else {
        String::from("ID non trovato")
    }
}


#[update(name="CheckNfts")]
fn check_nfts() -> () {
    let nfts = STATE.with(|state| {
        let mut state = state.borrow();
        state.nfts.clone()
    });

    ic_cdk::print("Start checking date of nfts...");
    for token_id in 0..nfts.len() {
        is_date_expired_nft(token_id as u64);
    }
    ic_cdk::print("--> terminate checking date of nfts <--");
}


#[update(name="checkDataExpiredNft")]
fn is_date_expired_nft(token_id:u64) -> bool {
    let id_input = usize::try_from(token_id);
    if id_input.is_err() {
        return false;
    }
    let token_id = id_input.unwrap();

    let nft = STATE.with(|state| {
        let mut state = state.borrow_mut();
        state
            .nfts
            .get(token_id).cloned()
    });

    if !nft.is_none() {
        let nft1 = nft.unwrap();

        if nft1.owner!=MGMT {
            let metadatapart = nft1.metadata.first();
            if metadatapart.is_none() {
                return false;
            }
            let metadata = metadatapart.unwrap();
            let key_val = &metadata.key_val_data;
            for (key, value) in key_val {
                if key.eq("expire_date") {
                    //let data = metadata.key_val_data.get("expire_date").unwrap();
                    let val2 = value.clone();

                    let val_int: String = if let MetadataVal::TextContent(c) = val2 {
                        c
                    } else {
                        String::from("01-01-1900")
                    };

                    let data_nft = chrono::naive::NaiveDate::parse_from_str(val_int.as_str(), "%d-%m-%Y").unwrap();
                    if data_nft.year() != i32::from(1900) {
                        let num_sec_from_ce = (ic_cdk::api::time() / 1000000000) as i64;
                        let date_now = NaiveDateTime::from_timestamp(num_sec_from_ce, 0).date();

                        ic_cdk::print("Today:");
                        ic_cdk::print(date_now.to_string());
                        ic_cdk::print("ndt's date:");
                        ic_cdk::print(data_nft.to_string());

                        if date_now > data_nft {
                            burn(token_id as u64);
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    return true;
                }
            }
        }
        return false;
    }
    return false;
}


// --------------
// burn interface
// --------------

#[update(name = "burnDip721")]
fn burn(token_id: u64) -> Result {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let nft = state
            .nfts
            .get_mut(usize::try_from(token_id)?)
            .ok_or(Error::InvalidTokenId)?;
        /*if nft.owner != api::caller() {
            Err(Error::Unauthorized)
        } else {*/
        nft.owner = MGMT;
        Ok(state.next_txid())
        //}
    })
}

#[derive(CandidType, Deserialize, Default)]
struct State {
    nfts: Vec<Nft>,
    custodians: HashSet<Principal>,
    operators: HashMap<Principal, HashSet<Principal>>, // owner to operators
    logo: Option<LogoResult>,
    name: String,
    symbol: String,
    txid: u128,
}

#[derive(CandidType, Deserialize, Clone)]
struct Nft {
    owner: Principal,
    approved: Option<Principal>,
    id: u64,
    metadata: MetadataDesc,
    //content: Vec<u8>,
    name: String,
}

type MetadataDesc = Vec<MetadataPart>;
type MetadataDescRef<'a> = &'a [MetadataPart];

#[derive(CandidType, Deserialize, Clone)]
struct MetadataPart {
    purpose: MetadataPurpose,
    key_val_data: HashMap<String, MetadataVal>,
    data: Vec<u8>,
}

#[derive(CandidType, Deserialize, PartialEq, Clone)]
enum MetadataPurpose {
    Preview,
    Rendered,
}

#[derive(CandidType, Deserialize)]
struct MintResult {
    token_id: u64,
    id: u128,
}

#[allow(clippy::enum_variant_names)]
#[derive(CandidType, Deserialize, Clone, Eq, PartialEq)]
enum MetadataVal {
    TextContent(String),
    BlobContent(Vec<u8>),
    NatContent(u128),
    Nat8Content(u8),
    Nat16Content(u16),
    Nat32Content(u32),
    Nat64Content(u64),
}

impl State {
    fn next_txid(&mut self) -> u128 {
        let txid = self.txid;
        self.txid += 1;
        txid
    }
}

#[derive(CandidType, Deserialize)]
enum InterfaceId {
    Approval,
    TransactionHistory,
    Mint,
    Burn,
    TransferNotification,
}

#[derive(CandidType, Deserialize)]
enum ConstrainedError {
    Unauthorized,
}

#[update]
fn set_name(name: String) -> Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.custodians.contains(&api::caller()) {
            state.name = name;
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    })
}

#[update]
fn set_symbol(sym: String) -> Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.custodians.contains(&api::caller()) {
            state.symbol = sym;
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    })
}

#[update]
fn set_logo(logo: Option<LogoResult>) -> Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.custodians.contains(&api::caller()) {
            state.logo = logo;
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    })
}

#[update]
fn set_custodian(user: Principal, custodian: bool) -> Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.custodians.contains(&api::caller()) {
            if custodian {
                state.custodians.insert(user);
            } else {
                state.custodians.remove(&user);
            }
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    })
}

#[query]
fn is_custodian(principal: Principal) -> bool {
    STATE.with(|state| state.borrow().custodians.contains(&principal))
}

//String, id account che esegue l'operazione
type IdStore = BTreeMap<String, Principal>;
type LicenseStore = BTreeMap<String, License>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct License {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub perpetual: bool,
    pub duration: u64,
    pub transferable: bool,
    pub transfer_commission: f64
    // pub keywords: Vec<String>,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct PurchaseInformations {
    pub license_id: String,
    pub price: String,
    pub date: String,
    pub to: String
}

thread_local! {
    static LICENSE_STORE: RefCell<LicenseStore> = RefCell::default();
    static ID_STORE: RefCell<IdStore> = RefCell::default();
}

#[query(name = "getSelf")]
fn get_self() -> License {
    // let id = ic_cdk::api::caller();
    // LICENSE_STORE.with(|license_store| {
    //     license_store
    //         .borrow()
    //         .get(&id)
    //         .cloned()
    //         .unwrap_or_else(|| License::default())
    // })
    return License::default()
}

#[query]
fn get_single(text: String) -> License {
    let text = text.to_lowercase();
    LICENSE_STORE.with(|license_store| {
        for (_, lic) in license_store.borrow().iter() {
            if lic.id.to_lowercase().eq(&text)
            {
                return lic.clone();
            }
        }
        return License::default();
    })
}

#[query]
fn get(name: String) -> License {
    LICENSE_STORE.with(|license_store| {
        license_store
            .borrow()
            .get(&name)
            .cloned()
            .unwrap_or_else(|| License::default())
    })
}

#[update]
fn update(license: License) -> String {
    let principal_id = ic_cdk::api::caller();
    let license_id = license.clone().id.clone();
    let id_previous_owner = ID_STORE.with(|id_store| {
        // id_store.borrow().contains_key(license_id.clone().as_ref())
        id_store.borrow().get(&license_id).cloned()
    });
    if id_previous_owner.is_some() && !id_previous_owner.unwrap().to_text().eq(&principal_id.to_text()){
        return String::from("Non puoi caricare un ID gi?? preso da un altro account")
    }
    ID_STORE.with(|id_store| {
        id_store
            .borrow_mut()
            .insert(license_id.clone(), principal_id);
    });
    LICENSE_STORE.with(|license_store| {
        license_store.borrow_mut().insert(license_id.clone(), license.clone());
    });
    if principal_id.clone() == Principal::anonymous() {
        let mut ret = String::from("Eri anonimo: ");
        ret.push_str(principal_id.to_text().as_str());
        return ret;
    }
    principal_id.to_text()
}


#[ic_cdk_macros::query]
fn list_products() -> Vec<License> {
    let mut licenze : Vec<License> = Vec::new();
    LICENSE_STORE.with(|license_store| {
        for (_, lic) in license_store.borrow().iter() {
            licenze.push(lic.clone())
        }
        return licenze.clone();
    })
}

#[query]
fn get_my_principal() -> String {
    let principal_id = ic_cdk::api::caller();
    return principal_id.to_text();
}

#[ic_cdk_macros::update]
fn confirm_purchase(signature: String, purchase_info: PurchaseInformations) -> String {
    let orig_data = signature_to_orig_data(signature.clone());
    let info = purchase_info.clone();
    let mut struct_content = info.license_id.to_owned();
    // format!("{:.2}", info.price)
    struct_content.push_str(info.price.as_str());
    struct_content.push_str(info.date.as_str());
    struct_content.push_str(info.to.as_str());

    if ! orig_data.eq(struct_content.as_str()) {
        return format!("Signature contains {} but found {} in struct", orig_data, struct_content);
    }

    let result = match verify_signature(signature.clone(), orig_data.clone()) {
        Ok(ris) => {ris}
        Err(_) => {false}
    };

    if result == true {

        let date = chrono::naive::NaiveDate::parse_from_str(purchase_info.date.as_str(), "%d-%m-%Y");
        //let date = chrono::naive::NaiveDate::parse_from_str(purchase_info.date.as_str(), "%Y-%m-%d");

        if date.is_err() {
            return String::from(purchase_info.date + " is not a valid date");
        }

        let date = date.unwrap();

        let this_license = get(purchase_info.license_id.clone());
        let license_price_str = format!("{:.2}", this_license.price);

        if ! license_price_str.eq(purchase_info.price.as_str()) {
            return format!("Expected {} as price but found {}", license_price_str, purchase_info.price.clone());
        }

        let date = date.checked_add_signed(Duration::days(this_license.duration as i64)).unwrap();

        let date_str = date.format("%d-%m-%Y").to_string();


        //genero NFT
        let principal = Principal::from_text(purchase_info.to).unwrap();
        let mut info = Vec::new();
        let mut metadata : HashMap<String, MetadataVal> = HashMap::new();
        if this_license.duration != 0 {
            metadata.insert(String::from("expire_date"), MetadataVal::TextContent(date_str));
        }
        metadata.insert(String::from("license_id"), MetadataVal::TextContent(purchase_info.license_id.clone()));
        let metadatapart = MetadataPart{
            purpose: MetadataPurpose::Preview,
            key_val_data: metadata,
            data: vec![]
        };
        info.push(metadatapart);
        mint(purchase_info.license_id.clone(), principal, info);
        return format!("The verification result for \"{}\" containing \"{}\" is {}, with id {} expires {}", signature, orig_data, result, purchase_info.license_id, date.format("%Y-%m-%d"))
    }
    return format!("The verification result for \"{}\" containing \"{}\" is {}, with id {}", signature, orig_data, result, purchase_info.license_id)
}
//https://docs.rs/ed25519-dalek/latest/ed25519_dalek/
//https://docs.rs/ed25519/latest/ed25519/
//https://github.com/RustCrypto/signatures/tree/master/ed25519
fn verify_signature(signature_str: String, signed_text: String) ->  Result<bool, SignatureError> {
    let pub_key = [9, 100, 165, 165, 48, 248, 113, 245, 88, 3, 54, 194, 65, 151, 60, 65, 247, 223, 186, 194, 77, 95, 190, 101, 70, 33, 94, 182, 111, 231, 45, 43];
    let public_key: PublicKey = PublicKey::from_bytes(&pub_key)?;
    // let signature_str = String::from("4f23d6692b340dbc92e163f5c271fe7d8f03e5836ec36eb474d8af6d2a22910de32a4284d6974302f6cfe9c716130cdc29dbeff8cb83171607516b700f28e30f41747461636b206174204461776e");
    let decoded_signature = signature_to_array(signature_str.clone());
    let signature_or_error = match decoded_signature {
        Ok(res) => {res},
        Err(_) => {return Err(Default::default());}
    };
    let signature = Signature::try_from(signature_or_error.as_ref())?;
    let message: &[u8] = signed_text.as_bytes();
    if public_key.verify(message, &signature).is_ok() {
        println!("verificato");
        Ok(true)
    } else {
        println!("FALSO");
        Ok(false)
    }
}


fn signature_to_array(hex: String) -> Result<[u8; 64], String> {
    let mut i = 0;
    let valori = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let mut result:  [u8; 64] = [0; 64];
    if hex.len() < 128 {
        return Err(String::from("String is too short"))
    }
    while i < hex.len() - 1 && i < 128 { //.try_into().unwrap()
        let el1; let el2;
        let opt1 = valori.iter().position(|&s| s == hex.chars().nth(i).unwrap());
        let opt2 = valori.iter().position(|&s| s == hex.chars().nth(i + 1).unwrap());
        if opt1.is_none() || opt2.is_none() {
            return Err(String::from("String contains invalid characters"));
        } else {
            el1 = opt1.unwrap();
            el2 = opt2.unwrap();
        }
        result[i/2] = (el1 * 16 + el2) as u8;
        i += 2;
    }
    return Ok(result);
}

fn signature_to_orig_data(hex: String) -> String {
    // let signed_size = (hex.len() - 128);
    let mut i = 128;
    let mut result = String::new();
    let valori = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    while i < hex.len() - 1 {
        let el1 = valori.iter().position(|&s| s == hex.chars().nth(i).unwrap()).unwrap();
        let el2 = valori.iter().position(|&s| s == hex.chars().nth(i + 1).unwrap()).unwrap();
        let character = (el1 * 16 + el2) as u8;
        result.push(character as char);
        i += 2;
    }
    return result;
}