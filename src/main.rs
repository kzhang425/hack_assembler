extern crate hack_assembler;
use hack_assembler::cmd;

fn main() {
    let commands = cmd::collect_env_args();
    cmd::interpret_args(&commands);
}
