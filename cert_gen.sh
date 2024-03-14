#!/bin/sh
mkdir -p $1
cd $1/

# Create unencrypted private key and a CSR (certificate signing request)
openssl req -newkey rsa:2048 -nodes -subj "/C=PL/CN=$2" -keyout key.pem -out key.csr

# Create a self-signed root CA
openssl req -x509 -sha256 -nodes -subj "/C=PL/CN=$2" -days 1825 -newkey rsa:2048 -keyout rootCA.key -out rootCA.crt

# Create file file.ext with the following content:
cat <<EOF >> file.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
subjectAltName = @alt_names
[alt_names]
DNS.1 = $2
IP.1 = $3
EOF

# Sign the CSR (`cert.pem`) with the root CA certificate and private key
openssl x509 -req -CA rootCA.crt -CAkey rootCA.key -in key.csr -out cert.pem -days 365 -CAcreateserial -extfile file.ext
