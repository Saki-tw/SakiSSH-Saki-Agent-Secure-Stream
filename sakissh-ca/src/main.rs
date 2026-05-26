use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "sakissh-ca", version = "5.0.0", about = "SakiAgentSSH CA Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new CA
    Init {
        #[arg(long, default_value = "Saki Studio Internal CA")]
        name: String,
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
    /// Issue a new certificate for an agent
    Issue {
        #[arg(long)]
        agent_name: String,
        #[arg(long)]
        ca_dir: Option<PathBuf>,
    },
}

use rcgen::{Certificate, CertificateParams, KeyPair, DistinguishedName, DnType, IsCa, BasicConstraints};
use std::fs;

fn init_ca(name: &str, out_dir: Option<PathBuf>) -> Result<()> {
    let dir = out_dir.unwrap_or_else(|| {
        dirs::home_dir().unwrap().join(".sakissh").join("tls")
    });
    fs::create_dir_all(&dir)?;

    let mut params = CertificateParams::new(vec![name.to_string()]).unwrap();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "Saki Studio");
    dn.push(DnType::CommonName, name);
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    
    // Valid for 10 years
    params.not_before = rcgen::date_time_ymd(2026, 1, 1);
    params.not_after = rcgen::date_time_ymd(2036, 1, 1);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    let ca_cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    fs::write(dir.join("ca.key"), private_key_pem)?;
    fs::write(dir.join("ca.crt"), ca_cert_pem)?;

    println!("CA initialized at {:?}", dir);
    Ok(())
}

fn issue_cert(agent_name: &str, ca_dir: Option<PathBuf>) -> Result<()> {
    let dir = ca_dir.unwrap_or_else(|| {
        dirs::home_dir().unwrap().join(".sakissh").join("tls")
    });
    
    let ca_cert_str = fs::read_to_string(dir.join("ca.crt"))
        .expect("CA cert not found. Run init first.");
    let ca_key_str = fs::read_to_string(dir.join("ca.key"))
        .expect("CA key not found. Run init first.");
    
    let ca_key = KeyPair::from_pem(&ca_key_str)?;
    let ca_cert_params = CertificateParams::from_ca_cert_pem(&ca_cert_str)?;
    let ca_cert = ca_cert_params.self_signed(&ca_key)?;

    let mut params = CertificateParams::new(vec![agent_name.to_string()]).unwrap();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "Saki Studio Agent");
    dn.push(DnType::CommonName, agent_name);
    params.distinguished_name = dn;
    
    params.not_before = rcgen::date_time_ymd(2026, 1, 1);
    params.not_after = rcgen::date_time_ymd(2027, 1, 1);

    let key_pair = KeyPair::generate()?;
    let cert = params.signed_by(&key_pair, &ca_cert, &ca_key)?;
    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    fs::write(dir.join(format!("{}.crt", agent_name)), cert_pem)?;
    fs::write(dir.join(format!("{}.key", agent_name)), private_key_pem)?;

    println!("Issued certificate for agent '{}' at {:?}", agent_name, dir);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { name, out_dir } => init_ca(&name, out_dir)?,
        Commands::Issue { agent_name, ca_dir } => issue_cert(&agent_name, ca_dir)?,
    }
    
    Ok(())
}
