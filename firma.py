import ed25519
import os


class Signature:

	def __init__(self):
		if os.path.exists("my-secret-key"):
			self.keydata = open("my-secret-key", "rb").read()
			self.privKey = ed25519.SigningKey(self.keydata)
			self.pubKey = self.privKey.get_verifying_key()
		else:
			self.privKey, self.pubKey = ed25519.create_keypair()
			open("my-secret-key", "wb").write(self.privKey.to_bytes())

	def print_keys(self):
		print("Private key (32 bytes):", self.privKey.to_ascii(encoding='hex'))
		print("Public key (32 bytes): ", self.pubKey.to_ascii(encoding='hex'))
		print("Public key (32 bytes) length: ", len(self.pubKey.to_bytes()))

		for b in self.pubKey.to_bytes():
			print(b, end=", ")

		print()

	def sign(self, message):
		# msg = b'Message for Ed'
		_signature = self.privKey.sign(message.encode(), encoding='hex')

		return _signature

	def verify(self, signed_text, text):
		try:
			self.pubKey.verify(signed_text, text.encode(), encoding='hex')
			return True
		except ed25519.BadSignatureError:
			return False


if __name__ == '__main__':
	import sys
	utility = Signature()
	utility.print_keys()
	signature = utility.sign(sys.argv[1] if len(sys.argv) == 2 else "Attack at Dawn.")
	print("Signature (64 bytes):", signature)
	print("Signature (64 bytes):", len(signature))

	for i in range(64):
		print(signature[i], end=", ")
	print()
	print()
	res = utility.verify(signature, "ciao")
	if res:
		print("The signature is valid.")
	else:
		print("Invalid signature!")
