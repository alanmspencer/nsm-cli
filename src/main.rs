extern crate clap;
use aws_nitro_enclaves_cose as cose;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use nsm_io::{AttestationDoc, ErrorCode, Request, Response};
use serde_bytes::ByteBuf;

fn main() {
    let matches = App::new("nsm-cli")
        .version("0.1.0")
        .about("Nitro Security Module Cli")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("describe-nsm")
                .about("Return capabilities and version of the connected NitroSecureModule"),
        )
        .subcommand(
            SubCommand::with_name("describe-pcr")
                .about("Read data from PlatformConfigurationRegister at some index")
                .arg(
                    Arg::with_name("index")
                        .short("i")
                        .long("index")
                        .required(true)
                        .takes_value(true)
                        .help("The PCR index (0..n)"),
                ),
        )
        .subcommand(
            SubCommand::with_name("attestation")
                .about("Create an AttestationDoc and sign it with it's private key to ensure authenticity")
                .arg(
                    Arg::with_name("userdata")
                        .short("ud")
                        .long("userdata")
                        .required(false)
                        .takes_value(true)
                        .help("Additional user data"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("describe-pcr", Some(sub_m)) => describe_pcr(sub_m),
        ("describe-nsm", Some(_)) => describe_nsm(),
        ("attestation", Some(sub_m)) => attestation(sub_m),
        _ => {}
    }
}

fn describe_pcr(sub_matches: &ArgMatches) {
    let index_arg = sub_matches.value_of("index").unwrap();
    let index_arg = index_arg.parse::<u16>().unwrap();

    let nsm_fd = nsm_driver::nsm_init();
    let request = Request::DescribePCR { index: index_arg };

    let response = nsm_driver::nsm_process_request(nsm_fd, request);

    let json = serde_json::to_string(&response);
    println!("{:?}", json.unwrap());

    nsm_driver::nsm_exit(nsm_fd);
}

fn describe_nsm() {
    let nsm_fd = nsm_driver::nsm_init();

    let request = Request::DescribeNSM {};
    let response = nsm_driver::nsm_process_request(nsm_fd, request);

    let json = serde_json::to_string(&response);
    println!("{:?}", json.unwrap());

    nsm_driver::nsm_exit(nsm_fd);
}

fn attestation(sub_matches: &ArgMatches) {
    let user_data = sub_matches.value_of("userdata").unwrap_or("");

    let nsm_fd = nsm_driver::nsm_init();

    let request = Request::Attestation {
        public_key: None,
        user_data: Some(ByteBuf::from(user_data)),
        nonce: None,
    };

    let result = match nsm_driver::nsm_process_request(nsm_fd, request) {
        Response::Attestation { document } => Ok(document),
        Response::Error(err) => Err(err),
        _ => Err(ErrorCode::InvalidResponse),
    };

    if result.is_err() {
        let json = serde_json::to_string(&result.unwrap_err());
        println!("{:?}", json.unwrap());
    } else {
        let cbor = result.unwrap() as Vec<u8>;
        let attestation_doc = attestation_decode(&cbor);
        let json = serde_json::to_string(&attestation_doc);
        println!("{:?}", json.unwrap());
    }

    nsm_driver::nsm_exit(nsm_fd);
}

fn attestation_decode(cbor: &Vec<u8>) -> AttestationDoc {
    let cose_doc = cose::CoseSign1::from_bytes(cbor).unwrap();
    let payload = cose_doc.get_payload(None).unwrap();

    AttestationDoc::from_binary(&payload).unwrap()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_attestation_decode() {
        let cbor = [
            132, 68, 161, 1, 56, 34, 160, 89, 16, 193, 169, 105, 109, 111, 100, 117, 108, 101, 95,
            105, 100, 120, 39, 105, 45, 48, 57, 101, 98, 49, 102, 56, 99, 48, 54, 53, 98, 55, 102,
            50, 101, 56, 45, 101, 110, 99, 48, 49, 55, 99, 57, 48, 49, 52, 101, 55, 50, 102, 57,
            100, 55, 56, 102, 100, 105, 103, 101, 115, 116, 102, 83, 72, 65, 51, 56, 52, 105, 116,
            105, 109, 101, 115, 116, 97, 109, 112, 27, 0, 0, 1, 124, 144, 20, 237, 236, 100, 112,
            99, 114, 115, 176, 0, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 88, 48, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 88, 48, 151, 231, 198, 144, 67,
            202, 175, 137, 92, 103, 120, 19, 38, 169, 227, 196, 215, 6, 7, 55, 88, 87, 118, 182,
            116, 250, 9, 65, 65, 94, 123, 17, 214, 160, 76, 213, 173, 188, 118, 145, 254, 223, 190,
            151, 5, 49, 163, 47, 4, 88, 48, 122, 19, 127, 215, 138, 72, 159, 167, 131, 161, 187,
            253, 250, 10, 209, 80, 204, 160, 128, 32, 223, 249, 189, 135, 204, 88, 245, 205, 72,
            172, 209, 80, 86, 223, 19, 58, 115, 53, 104, 118, 162, 126, 19, 66, 229, 101, 114, 214,
            5, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 88, 48, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 9, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10,
            88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 88, 48, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 14, 88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
            88, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 107, 99, 101, 114,
            116, 105, 102, 105, 99, 97, 116, 101, 89, 2, 127, 48, 130, 2, 123, 48, 130, 2, 1, 160,
            3, 2, 1, 2, 2, 16, 1, 124, 144, 20, 231, 47, 157, 120, 0, 0, 0, 0, 97, 108, 144, 159,
            48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3, 48, 129, 142, 49, 11, 48, 9, 6, 3, 85, 4,
            6, 19, 2, 85, 83, 49, 19, 48, 17, 6, 3, 85, 4, 8, 12, 10, 87, 97, 115, 104, 105, 110,
            103, 116, 111, 110, 49, 16, 48, 14, 6, 3, 85, 4, 7, 12, 7, 83, 101, 97, 116, 116, 108,
            101, 49, 15, 48, 13, 6, 3, 85, 4, 10, 12, 6, 65, 109, 97, 122, 111, 110, 49, 12, 48,
            10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49, 57, 48, 55, 6, 3, 85, 4, 3, 12, 48, 105,
            45, 48, 57, 101, 98, 49, 102, 56, 99, 48, 54, 53, 98, 55, 102, 50, 101, 56, 46, 117,
            115, 45, 101, 97, 115, 116, 45, 49, 46, 97, 119, 115, 46, 110, 105, 116, 114, 111, 45,
            101, 110, 99, 108, 97, 118, 101, 115, 48, 30, 23, 13, 50, 49, 49, 48, 49, 55, 50, 49,
            48, 55, 52, 51, 90, 23, 13, 50, 49, 49, 48, 49, 56, 48, 48, 48, 55, 52, 51, 90, 48,
            129, 147, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 19, 48, 17, 6, 3, 85, 4, 8,
            12, 10, 87, 97, 115, 104, 105, 110, 103, 116, 111, 110, 49, 16, 48, 14, 6, 3, 85, 4, 7,
            12, 7, 83, 101, 97, 116, 116, 108, 101, 49, 15, 48, 13, 6, 3, 85, 4, 10, 12, 6, 65,
            109, 97, 122, 111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49, 62, 48,
            60, 6, 3, 85, 4, 3, 12, 53, 105, 45, 48, 57, 101, 98, 49, 102, 56, 99, 48, 54, 53, 98,
            55, 102, 50, 101, 56, 45, 101, 110, 99, 48, 49, 55, 99, 57, 48, 49, 52, 101, 55, 50,
            102, 57, 100, 55, 56, 46, 117, 115, 45, 101, 97, 115, 116, 45, 49, 46, 97, 119, 115,
            48, 118, 48, 16, 6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 5, 43, 129, 4, 0, 34, 3, 98, 0,
            4, 217, 138, 8, 190, 198, 187, 36, 45, 248, 79, 67, 228, 13, 42, 188, 45, 64, 104, 29,
            37, 239, 244, 62, 216, 54, 83, 114, 26, 124, 162, 26, 114, 216, 67, 216, 60, 181, 123,
            47, 175, 124, 219, 124, 217, 45, 28, 2, 190, 245, 33, 99, 232, 193, 229, 84, 55, 192,
            84, 76, 129, 161, 78, 114, 193, 89, 73, 51, 210, 16, 18, 99, 29, 73, 80, 29, 244, 178,
            101, 17, 29, 31, 19, 99, 211, 254, 178, 252, 252, 206, 1, 118, 90, 75, 104, 33, 145,
            163, 29, 48, 27, 48, 12, 6, 3, 85, 29, 19, 1, 1, 255, 4, 2, 48, 0, 48, 11, 6, 3, 85,
            29, 15, 4, 4, 3, 2, 6, 192, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3, 3, 104, 0, 48,
            101, 2, 49, 0, 156, 206, 42, 35, 97, 226, 75, 43, 34, 121, 170, 234, 231, 85, 45, 152,
            74, 234, 47, 237, 162, 77, 213, 170, 135, 179, 105, 63, 234, 196, 182, 32, 27, 114,
            190, 206, 147, 133, 154, 218, 95, 65, 141, 175, 106, 175, 99, 180, 2, 48, 83, 123, 90,
            138, 59, 206, 19, 246, 219, 25, 49, 74, 98, 53, 174, 45, 20, 218, 253, 14, 1, 7, 13,
            252, 9, 45, 134, 52, 140, 60, 112, 43, 36, 83, 122, 210, 188, 11, 251, 210, 67, 79,
            240, 125, 151, 52, 161, 203, 104, 99, 97, 98, 117, 110, 100, 108, 101, 132, 89, 2, 21,
            48, 130, 2, 17, 48, 130, 1, 150, 160, 3, 2, 1, 2, 2, 17, 0, 249, 49, 117, 104, 27, 144,
            175, 225, 29, 70, 204, 180, 228, 231, 248, 86, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4,
            3, 3, 48, 73, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 15, 48, 13, 6, 3, 85,
            4, 10, 12, 6, 65, 109, 97, 122, 111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65,
            87, 83, 49, 27, 48, 25, 6, 3, 85, 4, 3, 12, 18, 97, 119, 115, 46, 110, 105, 116, 114,
            111, 45, 101, 110, 99, 108, 97, 118, 101, 115, 48, 30, 23, 13, 49, 57, 49, 48, 50, 56,
            49, 51, 50, 56, 48, 53, 90, 23, 13, 52, 57, 49, 48, 50, 56, 49, 52, 50, 56, 48, 53, 90,
            48, 73, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 15, 48, 13, 6, 3, 85, 4, 10,
            12, 6, 65, 109, 97, 122, 111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83,
            49, 27, 48, 25, 6, 3, 85, 4, 3, 12, 18, 97, 119, 115, 46, 110, 105, 116, 114, 111, 45,
            101, 110, 99, 108, 97, 118, 101, 115, 48, 118, 48, 16, 6, 7, 42, 134, 72, 206, 61, 2,
            1, 6, 5, 43, 129, 4, 0, 34, 3, 98, 0, 4, 252, 2, 84, 235, 166, 8, 193, 243, 104, 112,
            226, 154, 218, 144, 190, 70, 56, 50, 146, 115, 110, 137, 75, 255, 246, 114, 217, 137,
            68, 75, 80, 81, 229, 52, 164, 177, 246, 219, 227, 192, 188, 88, 26, 50, 183, 177, 118,
            7, 14, 222, 18, 214, 154, 63, 234, 33, 27, 102, 231, 82, 207, 125, 209, 221, 9, 95,
            111, 19, 112, 244, 23, 8, 67, 217, 220, 16, 1, 33, 228, 207, 99, 1, 40, 9, 102, 68,
            135, 201, 121, 98, 132, 48, 77, 197, 63, 244, 163, 66, 48, 64, 48, 15, 6, 3, 85, 29,
            19, 1, 1, 255, 4, 5, 48, 3, 1, 1, 255, 48, 29, 6, 3, 85, 29, 14, 4, 22, 4, 20, 144, 37,
            181, 13, 217, 5, 71, 231, 150, 195, 150, 250, 114, 157, 207, 153, 169, 223, 75, 150,
            48, 14, 6, 3, 85, 29, 15, 1, 1, 255, 4, 4, 3, 2, 1, 134, 48, 10, 6, 8, 42, 134, 72,
            206, 61, 4, 3, 3, 3, 105, 0, 48, 102, 2, 49, 0, 163, 127, 47, 145, 161, 201, 189, 94,
            231, 184, 98, 124, 22, 152, 210, 85, 3, 142, 31, 3, 67, 249, 91, 99, 169, 98, 140, 61,
            57, 128, 149, 69, 161, 30, 188, 191, 46, 59, 85, 216, 174, 238, 113, 180, 195, 214,
            173, 243, 2, 49, 0, 162, 243, 155, 22, 5, 178, 112, 40, 165, 221, 75, 160, 105, 181, 1,
            110, 101, 180, 251, 222, 143, 224, 6, 29, 106, 83, 25, 127, 156, 218, 245, 217, 67,
            188, 97, 252, 43, 235, 3, 203, 111, 238, 141, 35, 2, 243, 223, 246, 89, 2, 196, 48,
            130, 2, 192, 48, 130, 2, 69, 160, 3, 2, 1, 2, 2, 17, 0, 218, 81, 190, 226, 189, 226,
            87, 190, 142, 99, 204, 249, 242, 25, 76, 138, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3,
            3, 48, 73, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 15, 48, 13, 6, 3, 85, 4,
            10, 12, 6, 65, 109, 97, 122, 111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87,
            83, 49, 27, 48, 25, 6, 3, 85, 4, 3, 12, 18, 97, 119, 115, 46, 110, 105, 116, 114, 111,
            45, 101, 110, 99, 108, 97, 118, 101, 115, 48, 30, 23, 13, 50, 49, 49, 48, 49, 53, 48,
            48, 51, 50, 53, 54, 90, 23, 13, 50, 49, 49, 49, 48, 52, 48, 49, 51, 50, 53, 54, 90, 48,
            100, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 15, 48, 13, 6, 3, 85, 4, 10, 12,
            6, 65, 109, 97, 122, 111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49,
            54, 48, 52, 6, 3, 85, 4, 3, 12, 45, 54, 49, 55, 50, 49, 53, 98, 55, 48, 57, 55, 100,
            57, 100, 97, 56, 46, 117, 115, 45, 101, 97, 115, 116, 45, 49, 46, 97, 119, 115, 46,
            110, 105, 116, 114, 111, 45, 101, 110, 99, 108, 97, 118, 101, 115, 48, 118, 48, 16, 6,
            7, 42, 134, 72, 206, 61, 2, 1, 6, 5, 43, 129, 4, 0, 34, 3, 98, 0, 4, 127, 1, 216, 13,
            250, 76, 194, 7, 14, 135, 239, 222, 89, 254, 148, 231, 10, 34, 203, 146, 197, 234, 211,
            245, 186, 199, 221, 76, 15, 164, 120, 177, 205, 177, 2, 107, 213, 120, 46, 130, 60,
            238, 78, 50, 69, 193, 188, 19, 143, 113, 146, 146, 24, 184, 236, 194, 42, 218, 223, 91,
            10, 49, 34, 8, 197, 58, 19, 12, 29, 168, 211, 227, 121, 205, 138, 212, 157, 177, 78,
            21, 176, 82, 227, 15, 163, 1, 126, 82, 61, 129, 215, 151, 244, 64, 41, 53, 163, 129,
            213, 48, 129, 210, 48, 18, 6, 3, 85, 29, 19, 1, 1, 255, 4, 8, 48, 6, 1, 1, 255, 2, 1,
            2, 48, 31, 6, 3, 85, 29, 35, 4, 24, 48, 22, 128, 20, 144, 37, 181, 13, 217, 5, 71, 231,
            150, 195, 150, 250, 114, 157, 207, 153, 169, 223, 75, 150, 48, 29, 6, 3, 85, 29, 14, 4,
            22, 4, 20, 203, 62, 133, 86, 159, 188, 133, 62, 1, 106, 247, 224, 12, 64, 7, 205, 25,
            134, 216, 234, 48, 14, 6, 3, 85, 29, 15, 1, 1, 255, 4, 4, 3, 2, 1, 134, 48, 108, 6, 3,
            85, 29, 31, 4, 101, 48, 99, 48, 97, 160, 95, 160, 93, 134, 91, 104, 116, 116, 112, 58,
            47, 47, 97, 119, 115, 45, 110, 105, 116, 114, 111, 45, 101, 110, 99, 108, 97, 118, 101,
            115, 45, 99, 114, 108, 46, 115, 51, 46, 97, 109, 97, 122, 111, 110, 97, 119, 115, 46,
            99, 111, 109, 47, 99, 114, 108, 47, 97, 98, 52, 57, 54, 48, 99, 99, 45, 55, 100, 54,
            51, 45, 52, 50, 98, 100, 45, 57, 101, 57, 102, 45, 53, 57, 51, 51, 56, 99, 98, 54, 55,
            102, 56, 52, 46, 99, 114, 108, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3, 3, 105, 0,
            48, 102, 2, 49, 0, 178, 18, 111, 197, 248, 88, 245, 82, 16, 194, 173, 101, 34, 225, 10,
            162, 36, 75, 102, 59, 114, 69, 226, 115, 125, 30, 219, 163, 169, 177, 224, 26, 160, 77,
            47, 138, 149, 234, 209, 51, 198, 77, 78, 124, 169, 8, 93, 61, 2, 49, 0, 193, 111, 226,
            108, 252, 80, 194, 60, 200, 255, 168, 36, 32, 103, 138, 60, 113, 191, 151, 223, 104,
            111, 78, 253, 53, 126, 117, 113, 118, 138, 134, 37, 143, 126, 238, 221, 64, 146, 203,
            107, 238, 236, 222, 104, 65, 160, 84, 102, 89, 3, 24, 48, 130, 3, 20, 48, 130, 2, 154,
            160, 3, 2, 1, 2, 2, 16, 111, 99, 64, 104, 221, 123, 165, 18, 195, 79, 117, 187, 115,
            182, 110, 137, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3, 48, 100, 49, 11, 48, 9, 6,
            3, 85, 4, 6, 19, 2, 85, 83, 49, 15, 48, 13, 6, 3, 85, 4, 10, 12, 6, 65, 109, 97, 122,
            111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49, 54, 48, 52, 6, 3, 85,
            4, 3, 12, 45, 54, 49, 55, 50, 49, 53, 98, 55, 48, 57, 55, 100, 57, 100, 97, 56, 46,
            117, 115, 45, 101, 97, 115, 116, 45, 49, 46, 97, 119, 115, 46, 110, 105, 116, 114, 111,
            45, 101, 110, 99, 108, 97, 118, 101, 115, 48, 30, 23, 13, 50, 49, 49, 48, 49, 55, 49,
            55, 53, 48, 53, 49, 90, 23, 13, 50, 49, 49, 48, 50, 51, 49, 56, 53, 48, 53, 48, 90, 48,
            129, 137, 49, 60, 48, 58, 6, 3, 85, 4, 3, 12, 51, 99, 50, 56, 56, 102, 50, 102, 51, 97,
            57, 51, 55, 57, 53, 53, 51, 46, 122, 111, 110, 97, 108, 46, 117, 115, 45, 101, 97, 115,
            116, 45, 49, 46, 97, 119, 115, 46, 110, 105, 116, 114, 111, 45, 101, 110, 99, 108, 97,
            118, 101, 115, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49, 15, 48, 13, 6,
            3, 85, 4, 10, 12, 6, 65, 109, 97, 122, 111, 110, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2,
            85, 83, 49, 11, 48, 9, 6, 3, 85, 4, 8, 12, 2, 87, 65, 49, 16, 48, 14, 6, 3, 85, 4, 7,
            12, 7, 83, 101, 97, 116, 116, 108, 101, 48, 118, 48, 16, 6, 7, 42, 134, 72, 206, 61, 2,
            1, 6, 5, 43, 129, 4, 0, 34, 3, 98, 0, 4, 100, 224, 19, 248, 157, 252, 77, 218, 39, 53,
            35, 183, 235, 215, 224, 197, 104, 174, 67, 148, 66, 187, 59, 175, 143, 126, 150, 241,
            239, 229, 18, 200, 138, 227, 193, 182, 185, 162, 255, 227, 140, 103, 120, 22, 182, 246,
            94, 60, 139, 170, 135, 111, 147, 153, 208, 128, 162, 192, 102, 171, 127, 31, 181, 105,
            4, 169, 160, 33, 162, 229, 153, 117, 164, 113, 12, 250, 164, 162, 74, 68, 132, 111,
            140, 199, 102, 131, 217, 56, 63, 182, 65, 193, 11, 38, 199, 179, 163, 129, 234, 48,
            129, 231, 48, 18, 6, 3, 85, 29, 19, 1, 1, 255, 4, 8, 48, 6, 1, 1, 255, 2, 1, 1, 48, 31,
            6, 3, 85, 29, 35, 4, 24, 48, 22, 128, 20, 203, 62, 133, 86, 159, 188, 133, 62, 1, 106,
            247, 224, 12, 64, 7, 205, 25, 134, 216, 234, 48, 29, 6, 3, 85, 29, 14, 4, 22, 4, 20,
            115, 118, 83, 21, 2, 187, 195, 242, 83, 120, 223, 216, 5, 247, 26, 3, 99, 128, 150, 14,
            48, 14, 6, 3, 85, 29, 15, 1, 1, 255, 4, 4, 3, 2, 1, 134, 48, 129, 128, 6, 3, 85, 29,
            31, 4, 121, 48, 119, 48, 117, 160, 115, 160, 113, 134, 111, 104, 116, 116, 112, 58, 47,
            47, 99, 114, 108, 45, 117, 115, 45, 101, 97, 115, 116, 45, 49, 45, 97, 119, 115, 45,
            110, 105, 116, 114, 111, 45, 101, 110, 99, 108, 97, 118, 101, 115, 46, 115, 51, 46,
            117, 115, 45, 101, 97, 115, 116, 45, 49, 46, 97, 109, 97, 122, 111, 110, 97, 119, 115,
            46, 99, 111, 109, 47, 99, 114, 108, 47, 102, 57, 50, 57, 57, 49, 55, 57, 45, 48, 102,
            97, 97, 45, 52, 53, 50, 101, 45, 98, 50, 97, 48, 45, 57, 51, 53, 99, 56, 99, 48, 51,
            102, 50, 50, 97, 46, 99, 114, 108, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3, 3, 104,
            0, 48, 101, 2, 49, 0, 143, 63, 67, 166, 191, 85, 121, 192, 28, 194, 78, 67, 201, 207,
            37, 165, 86, 171, 5, 139, 162, 51, 194, 3, 24, 248, 123, 155, 92, 54, 132, 157, 192,
            161, 138, 106, 233, 15, 68, 217, 191, 176, 121, 237, 148, 140, 90, 127, 2, 48, 123, 65,
            142, 136, 44, 213, 92, 60, 135, 228, 205, 180, 83, 167, 246, 248, 152, 33, 154, 34,
            160, 30, 48, 175, 103, 112, 136, 146, 203, 188, 131, 126, 233, 33, 144, 4, 11, 153, 96,
            21, 226, 89, 167, 42, 56, 158, 24, 219, 89, 2, 131, 48, 130, 2, 127, 48, 130, 2, 5,
            160, 3, 2, 1, 2, 2, 21, 0, 248, 66, 124, 191, 188, 14, 16, 231, 102, 145, 218, 189,
            253, 29, 251, 86, 145, 202, 58, 68, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3, 48,
            129, 137, 49, 60, 48, 58, 6, 3, 85, 4, 3, 12, 51, 99, 50, 56, 56, 102, 50, 102, 51, 97,
            57, 51, 55, 57, 53, 53, 51, 46, 122, 111, 110, 97, 108, 46, 117, 115, 45, 101, 97, 115,
            116, 45, 49, 46, 97, 119, 115, 46, 110, 105, 116, 114, 111, 45, 101, 110, 99, 108, 97,
            118, 101, 115, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49, 15, 48, 13, 6,
            3, 85, 4, 10, 12, 6, 65, 109, 97, 122, 111, 110, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2,
            85, 83, 49, 11, 48, 9, 6, 3, 85, 4, 8, 12, 2, 87, 65, 49, 16, 48, 14, 6, 3, 85, 4, 7,
            12, 7, 83, 101, 97, 116, 116, 108, 101, 48, 30, 23, 13, 50, 49, 49, 48, 49, 55, 50, 49,
            48, 48, 53, 49, 90, 23, 13, 50, 49, 49, 48, 49, 56, 50, 49, 48, 48, 53, 49, 90, 48,
            129, 142, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 19, 48, 17, 6, 3, 85, 4, 8,
            12, 10, 87, 97, 115, 104, 105, 110, 103, 116, 111, 110, 49, 16, 48, 14, 6, 3, 85, 4, 7,
            12, 7, 83, 101, 97, 116, 116, 108, 101, 49, 15, 48, 13, 6, 3, 85, 4, 10, 12, 6, 65,
            109, 97, 122, 111, 110, 49, 12, 48, 10, 6, 3, 85, 4, 11, 12, 3, 65, 87, 83, 49, 57, 48,
            55, 6, 3, 85, 4, 3, 12, 48, 105, 45, 48, 57, 101, 98, 49, 102, 56, 99, 48, 54, 53, 98,
            55, 102, 50, 101, 56, 46, 117, 115, 45, 101, 97, 115, 116, 45, 49, 46, 97, 119, 115,
            46, 110, 105, 116, 114, 111, 45, 101, 110, 99, 108, 97, 118, 101, 115, 48, 118, 48, 16,
            6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 5, 43, 129, 4, 0, 34, 3, 98, 0, 4, 161, 32, 191,
            136, 76, 195, 51, 108, 185, 134, 67, 173, 152, 188, 177, 240, 9, 25, 237, 210, 59, 180,
            98, 71, 80, 35, 54, 213, 151, 117, 117, 246, 10, 72, 176, 53, 85, 58, 122, 168, 132,
            33, 229, 183, 80, 64, 116, 79, 221, 204, 126, 25, 243, 59, 6, 169, 61, 186, 60, 38,
            225, 72, 8, 136, 122, 87, 98, 140, 206, 145, 224, 71, 131, 147, 21, 164, 99, 16, 181,
            30, 56, 47, 183, 59, 28, 82, 185, 35, 129, 85, 153, 251, 45, 81, 202, 21, 163, 38, 48,
            36, 48, 18, 6, 3, 85, 29, 19, 1, 1, 255, 4, 8, 48, 6, 1, 1, 255, 2, 1, 0, 48, 14, 6, 3,
            85, 29, 15, 1, 1, 255, 4, 4, 3, 2, 2, 4, 48, 10, 6, 8, 42, 134, 72, 206, 61, 4, 3, 3,
            3, 104, 0, 48, 101, 2, 49, 0, 233, 218, 249, 75, 225, 149, 142, 176, 11, 84, 228, 206,
            248, 215, 204, 70, 69, 252, 189, 123, 195, 14, 249, 104, 2, 30, 1, 255, 210, 36, 10,
            138, 213, 71, 54, 92, 252, 202, 160, 193, 187, 244, 225, 39, 214, 46, 43, 64, 2, 48,
            17, 91, 145, 185, 231, 88, 29, 230, 218, 163, 191, 185, 54, 104, 25, 179, 74, 161, 176,
            105, 114, 249, 94, 188, 41, 136, 42, 202, 46, 188, 208, 141, 224, 166, 30, 72, 27, 77,
            43, 200, 33, 11, 217, 54, 110, 55, 52, 136, 106, 112, 117, 98, 108, 105, 99, 95, 107,
            101, 121, 246, 105, 117, 115, 101, 114, 95, 100, 97, 116, 97, 64, 101, 110, 111, 110,
            99, 101, 246, 88, 96, 93, 92, 32, 232, 117, 207, 55, 44, 77, 113, 77, 187, 2, 164, 120,
            107, 222, 151, 3, 243, 210, 246, 242, 89, 224, 133, 38, 12, 33, 253, 111, 175, 118,
            155, 60, 36, 189, 8, 125, 11, 3, 114, 187, 181, 177, 96, 219, 230, 90, 165, 15, 252,
            70, 191, 160, 30, 7, 142, 154, 37, 35, 132, 233, 230, 139, 170, 201, 54, 223, 127, 21,
            44, 70, 62, 49, 60, 2, 89, 153, 146, 196, 93, 20, 195, 54, 28, 89, 184, 237, 181, 152,
            114, 41, 7, 190, 210,
        ];
        let attestation_doc = attestation_decode(&cbor.to_vec());
        assert_eq!(
            attestation_doc.module_id,
            "i-09eb1f8c065b7f2e8-enc017c9014e72f9d78"
        );
        //println!("{:?}", attestation_doc);
    }
}
