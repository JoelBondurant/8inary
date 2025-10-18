COUNTRY="US"
STATE="Texas"
CITY="Austin"
ORG="8inary"
OU="dev"
CN="localhost"
CERT_PATH="/etc/letsencrypt/live/8inary.com/"

openssl ecparam -genkey -name secp256r1 -out privkey.pem
openssl req -new -x509 -sha256 -key privkey.pem -out fullchain.pem \
	-days 365 -nodes -subj "/C=US/ST=Texas/L=Austin/O=8inary/OU=dev/CN=localhost"

sudo mkdir -p $CERT_PATH
sudo mv privkey.pem $CERT_PATH
sudo mv fullchain.pem $CERT_PATH

# sudo chown -R intergate: /etc/letsencrypt/live/8inary.com
# sudo chown :intergate /etc/letsencrypt
# sudo chown -R :intergate /etc/letsencrypt/live /etc/letsencrypt/archive
# sudo chmod g+rx /etc/letsencrypt
# sudo chmod -R g+rx /etc/letsencrypt/live /etc/letsencrypt/archive
# Todo: add certbot post-renewal hook to copy certs to a non-pathalogical path...
