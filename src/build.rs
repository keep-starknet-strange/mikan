
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/block.proto"], &["src/"])
}
