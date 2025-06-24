use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

pub async fn load_ssl_certificate(certificate_path:String,certificate_key_path:String) -> SslAcceptorBuilder {
    // Load certificate
    // let cert_file = &mut BufReader::new(File::open(certificate_path).unwrap());
    // let key_file = &mut BufReader::new(File::open(certificate_key_path).unwrap());

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(certificate_key_path, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(certificate_path).unwrap();

    builder

}