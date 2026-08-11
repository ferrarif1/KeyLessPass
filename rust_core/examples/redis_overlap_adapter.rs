use keylesspass_core::domain::{
    apply_authentication_probe, transition_rotation, AuthenticationProbe,
    CredentialDescriptionRecord, EncodingDescriptor, ProbeVerdict, ProbedCredential,
    RotationContract, RotationEvent, RotationState,
};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("KEYLESSPASS_REDIS_TEST_ADDR").map_err(|_| {
        "set KEYLESSPASS_REDIS_TEST_ADDR to an isolated Redis test instance, for example 127.0.0.1:16379"
    })?;
    let user = format!("keylesspass-test-{}", Uuid::new_v4().simple());
    let old_password = format!("Old-{}!", Uuid::new_v4().simple());
    let new_password = format!("New-{}!", Uuid::new_v4().simple());

    command(
        &address,
        &[
            "ACL",
            "SETUSER",
            &user,
            "reset",
            "on",
            "+ping",
            &format!(">{old_password}"),
        ],
    )?;

    let scenario = run_overlap_scenario(&address, &user, &old_password, &new_password);
    let cleanup = command(&address, &["ACL", "DELUSER", &user]);
    scenario?;
    cleanup?;
    println!("Redis overlap adapter: BOTH -> revoke old -> NEW_ONLY verified");
    Ok(())
}

fn run_overlap_scenario(
    address: &str,
    user: &str,
    old_password: &str,
    new_password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let active = CredentialDescriptionRecord::new(
        Uuid::new_v4(),
        1,
        1,
        "Redis ACL",
        address,
        user,
        "",
        EncodingDescriptor::default(),
    );
    let mut pending = CredentialDescriptionRecord::rotation_from_with_contract(
        &active,
        EncodingDescriptor::default(),
        RotationContract::OverlapThenRevoke,
    );

    command(
        address,
        &["ACL", "SETUSER", user, &format!(">{new_password}")],
    )?;
    record_probe(
        &mut pending,
        ProbedCredential::New,
        authenticate(address, user, new_password),
        address,
    )?;
    record_probe(
        &mut pending,
        ProbedCredential::Old,
        authenticate(address, user, old_password),
        address,
    )?;
    if pending.rotation_state != RotationState::OverlapEstablished {
        return Err("Redis did not establish the expected BOTH state".into());
    }

    transition_rotation(&mut pending, RotationEvent::RequestOldRevocation)?;
    command(
        address,
        &["ACL", "SETUSER", user, &format!("<{old_password}")],
    )?;
    record_probe(
        &mut pending,
        ProbedCredential::New,
        authenticate(address, user, new_password),
        address,
    )?;
    record_probe(
        &mut pending,
        ProbedCredential::Old,
        authenticate(address, user, old_password),
        address,
    )?;
    if pending.rotation_state != RotationState::RemoteConfirmed {
        return Err("Redis did not converge to NEW_ONLY after old-password revocation".into());
    }
    Ok(())
}

fn record_probe(
    record: &mut CredentialDescriptionRecord,
    credential: ProbedCredential,
    succeeded: bool,
    endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let verdict = if succeeded {
        ProbeVerdict::Success
    } else {
        ProbeVerdict::ConclusiveFailure
    };
    apply_authentication_probe(
        record,
        AuthenticationProbe::now(credential, verdict, endpoint),
    )?;
    Ok(())
}

fn authenticate(address: &str, user: &str, password: &str) -> bool {
    command(address, &["AUTH", user, password]).is_ok()
}

fn command(address: &str, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(address)?;
    write!(stream, "*{}\r\n", args.len())?;
    for arg in args {
        write!(stream, "${}\r\n{}\r\n", arg.len(), arg)?;
    }
    stream.flush()?;
    read_response(&mut BufReader::new(stream))
}

fn read_response(reader: &mut BufReader<TcpStream>) -> Result<String, Box<dyn std::error::Error>> {
    let mut prefix = [0_u8; 1];
    reader.read_exact(&mut prefix)?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let value = line.trim_end_matches(['\r', '\n']);
    match prefix[0] {
        b'+' | b':' => Ok(value.to_string()),
        b'-' => Err(format!("Redis error: {value}").into()),
        _ => Err("unsupported Redis response type".into()),
    }
}
