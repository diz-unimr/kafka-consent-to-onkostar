use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help(true))]
pub(crate) struct Cli {
    #[arg(
        long,
        env = "KAFKA_BOOTSTRAP_SERVERS",
        default_value = "kafka:9094",
        help = "Kafka Bootstrap Server"
    )]
    pub bootstrap_servers: String,
    #[arg(
        long,
        env = "KAFKA_TOPIC",
        default_value = "consent-json-idat",
        help = "Kafka Topic"
    )]
    pub topic: String,
    #[arg(
        long,
        env = "KAFKA_GROUP_ID",
        default_value = "kafka-consent-to-onkostar",
        help = "Kafka Group ID"
    )]
    pub group_id: String,
    #[arg(long, env = "ONKOSTAR_URI", help = "Onkostar URI for API requests")]
    #[arg(
        long,
        env = "KAFKA_SSL_CA_FILE",
        help = "CA file for SSL connection to Kafka"
    )]
    pub ssl_ca_file: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_CERT_FILE",
        help = "Certificate file for SSL connection to Kafka"
    )]
    pub ssl_cert_file: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_KEY_FILE",
        help = "Key file for SSL connection to Kafka"
    )]
    pub ssl_key_file: Option<String>,
    #[arg(long, env = "KAFKA_SSL_KEY_PASSWORD", help = "The SSL key password")]
    pub ssl_key_password: Option<String>,
    pub onkostar_uri: String,
    #[arg(long, env = "ONKOSTAR_USERNAME", help = "Onkostar Username")]
    pub onkostar_username: Option<String>,
    #[arg(long, env = "ONKOSTAR_PASSWORD", help = "Onkostar Password")]
    pub onkostar_password: Option<String>,
}
