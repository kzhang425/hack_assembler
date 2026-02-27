// File to hold constants or static strings.

pub const FILE_OPEN_ERROR: &str = "Unable to open file. Please check if the file exists."; // File open error message.
pub const FILE_CREATE_ERROR: &str = "Unable to write to file. Please check if a valid file path is provided."; // File create error message.

// Assembler
pub const ASSEMBLER_APP: &str = "ASM";
pub const ASSEMBLER_HELP: &str = "Use the arguments \"-i\" and \"-o\" followed by a file path to ingest a Hack assembly file and convert it to binary.";
pub const ASSEMBLER_DEF_EXTENSION: &str = "_out.hack";
pub const ASSEMBLER_EMPTY_LINE: &str = "Error: Did not expect empty line.";
pub const SCREEN_LOC: isize = 16384;
pub const KBD_LOC: isize = 24576;
pub const LABEL_ERR: &str = "Label name is invalid.";