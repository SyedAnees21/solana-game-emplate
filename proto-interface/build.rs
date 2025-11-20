use std::path::PathBuf;

fn main() {
    #[cfg(feature = "proto")]
    {
        // Path to the .proto sources (relative to this crate)
        let proto_dir = PathBuf::from("../protos");
    
        let protos = &[
            proto_dir.join("handshake.proto"),
            proto_dir.join("attributes.proto"),
            proto_dir.join("incoming.proto"),
            proto_dir.join("messages.proto"),
            proto_dir.join("outgoing.proto"),
        ];
    
        for proto in protos {
            println!("cargo:rerun-if-changed={}", proto.display());
        }
    
        match prost_build::Config::new()
            .out_dir(PathBuf::from("src/external/"))
            .compile_well_known_types()
            .compile_protos(protos, &[proto_dir])
        {
            Ok(_) => println!("Protobufs compiled successfully."),
            Err(e) => eprintln!("Failed to compile protobufs: {}", e),
        }
    }
}
