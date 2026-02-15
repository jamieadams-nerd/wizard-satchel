use nom::{
    bytes::complete::{tag, take_while1},
    IResult,
};
use std::io;

/// NIST 800-53 AC-4 / NSA RTB (Redundant)
/// We define two completely different ways to extract the 'Type' from a context:
/// "user:role:type:level"

// --- PATH A: The "nom" Combinator Parser (Declarative) ---
fn parse_type_nom(input: &str) -> IResult<&str, &str> {
    // Skip user: and role: then take the type
    let (input, _) = take_while1(|c| c != ':')(input)?; // user
    let (input, _) = tag(":")(input)?;
    let (input, _) = take_while1(|c| c != ':')(input)?; // role
    let (input, _) = tag(":")(input)?;
    let (input, type_str) = take_while1(|c| c != ':')(input)?; // type
    Ok((input, type_str))
}

// --- PATH B: The Manual Byte-Iterator (Imperative/C-style) ---
fn parse_type_manual(input: &str) -> io::Result<&str> {
    let mut parts = input.split(':');
    let _user = parts.next().ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing user"))?;
    let _role = parts.next().ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing role"))?;
    let type_str = parts.next().ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing type"))?;
    Ok(type_str)
}

/// The RTB "RAIN" Validator
pub fn validate_context_type(raw_context: &str) -> io::Result<&str> {
    // Execute Path A
    let (_, type_a) = parse_type_nom(raw_context)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Nom parser failed"))?;

    // Execute Path B
    let type_b = parse_type_manual(raw_context)?;

    // NIST 800-53 SI-7: Software Integrity Check
    // If the high-level combinator and the low-level iterator disagree, 
    // we have a logic fault. Fail Closed.
    if type_a != type_b {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "RTB Redundancy Failure: Parser Mismatch Detected"
        ));
    }

    Ok(type_a)
}

