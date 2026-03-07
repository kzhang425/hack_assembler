use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::cmd;
use crate::consts;

/// Core logic to actually execute the assembly given the arguments provided.
pub fn do_assembly(args: &Vec<String>) {
    let mut input_file_path = "";
    let mut output_file_path = "";
    let mut input_provided = false;
    let mut output_provided = false;
    let mut flag;

    if args.len() <= 2 {
        instructions();
        return;
    }

    for index in 2..args.len() {
        flag = args[index].to_ascii_uppercase();
        if flag == "-I" {
            let field = args.get(index + 1);
            if let Some(fp) = field {
                input_file_path = fp;
                input_provided = true;
            }
        } else if flag == "-O" {
            let field = args.get(index + 1);
            if let Some(fp) = field {
                output_file_path = fp;
                output_provided = true;
            }
        }
    }

    // If we couldn't identify an input and output filepath even before validation, just quit.
    if !input_provided {
        return;
    }

    // See if input file is valid.
    let input_file: File;
    match cmd::handle_file_path(input_file_path, false) {
        Ok(in_file) => input_file = in_file,
        Err(err_str) => {
            println!("{}", err_str);
            return;
        },
    }

    // Do we have a valid file path to spit out to? If not, just try and use the input filepath and rename it.
    let mut temp_str: String;
    if !output_provided {
        temp_str = remove_last_delimiter(input_file_path, '.');
        temp_str.push_str(consts::ASSEMBLER_DEF_EXTENSION);
        output_file_path = temp_str.as_str();
    }

    // Now that we have inputs and outputs, input_file is a file ready to read.
    let mut output_file: File;
    match cmd::handle_file_path(output_file_path, true) {
        Ok(out) => output_file = out,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Create assembler object to handle the processing steps.
    let mut assembler = Assembler::init_from_file(input_file); // Consume the input file into an Assembler object.
    if let Err(e) = assembler.process_labels() {
        println!("{}", e);
        return;
    }

    let hack_output: Vec<String>;
    match assembler.translate_to_vec_str() {
        Err(e) => {
            println!("{}", e);
            return;
        }

        Ok(out) => {
            hack_output = out;
        }
    }

    for line in hack_output {
        output_file.write(line.as_bytes()).unwrap();
        output_file.write("\n".as_bytes()).unwrap();
    }
}


/// A struct that contains the program data and symbol table to process into the Hack binary language.
/// 
/// # Example
/// ```
/// let mut asm = Assembler::init_from_file(f); // Where f is a File object
/// asm.process_labels()?;
/// let output = asm.translate_to_vec_str(); // Output is a Vec<String>
/// ```
struct Assembler {
    program: Instructions,
    sym_tbl: SymbolTable,
}

impl Assembler {
    /// Using an input file, initializes a symbol table and the vector of instructions to parse. Can assume that there are no blanks and no comments in Instructions after this call.
    pub fn init_from_file(f: File) -> Self {
        let sym_tbl = SymbolTable::init();
        let program = Instructions::import_from_file(f);
        Self {
            program,
            sym_tbl,
        }
    }

    /// Call this after initializing the assembler object to handle labels and strip them out of the main program to be loaded in ROM. Adds these labels to the symbol table.
    pub fn process_labels(&mut self) -> Result<(), String> {
        let sym_import_vec = self.program.parse_and_remove_labels()?;
        self.sym_tbl.mass_add_to_symbol_table(sym_import_vec);
        Ok(())
    }

    pub fn translate_to_vec_str(mut self) -> Result<Vec<String>, String> {
        let mut output = Vec::<String>::new();
        for line in 0..self.program.len() {
            let line_result = self.program.process_one_instruction(line, &mut self.sym_tbl);
            match line_result {
                Ok(s) => {
                    output.push(s);
                }

                Err(e) => {
                    let error_str = format!("Error occurred in command {}: {}", self.program.get(line).unwrap(), e);
                    return Err(error_str);
                }
            }
        }
        Ok(output)
    }
}

pub struct SymbolTable {
    main_table: HashMap<String, isize>,
    auto_assign_id: isize,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            main_table: HashMap::new(),
            auto_assign_id: 0,
        }
    }

    /// The main tag to initialize a SymbolTable object. This handles adding some of the hard-coded values.
    pub fn init() -> Self {
        let mut out = Self::new();
        out.add_hard_coded_symbols();
        out
    }

    /// Attempts to get the value from the symbol table if it exists. If so, returns the integer value.
    pub fn get(&self, input: &str) -> Option<isize> {
        self.main_table.get(input).copied()
    }

    /// Internal function to increment the auto_assign_id field when calling functions that should.
    fn inc_auto_assign_id(&mut self) {
        self.auto_assign_id += 1;
    }

    fn does_symbol_exist(&self, key: &str) -> bool {
        self.main_table.contains_key(key)
    }

    /// If need to define a symbol and value, use this function.
    pub fn add_to_symbol_table(&mut self, sym: String, val: isize) {
        if !self.does_symbol_exist(&sym) {
            self.main_table.insert(sym, val);
        }
    }

    pub fn mass_add_to_symbol_table(&mut self, input: Vec<(String, isize)>) {
        for (key, val) in input {
            self.add_to_symbol_table(key, val);
        }
    }

    /// Uses the available ID in auto_assign_id and assigns it to a new symbol, then increments it. Useful when needing to assign
    /// arbitrary values.
    pub fn add_to_symbol_table_auto(&mut self, sym: String) {
        if self.does_symbol_exist(&sym) {
            return;
        }
        let id_to_use = self.auto_assign_id;
        self.add_to_symbol_table(sym, id_to_use);
        self.inc_auto_assign_id();
    }

    /// Adds the values for R0 through R15, as well as the mnemonics of SCREEN and KBD.
    fn add_hard_coded_symbols(&mut self) {
        let registers_prefix = "R".to_string();
        let mut register_name: String;
        for i in 0..16 {
            let int_as_str = i.to_string();
            register_name = registers_prefix.clone();
            register_name.push_str(&int_as_str);
            self.add_to_symbol_table_auto(register_name);
        }
        self.add_to_symbol_table("SCREEN".to_string(), consts::SCREEN_LOC);
        self.add_to_symbol_table("KBD".to_string(), consts::KBD_LOC);
    }
}

/// Struct built to house the program data for processing and eventual translation into the Hack machine code.
struct Instructions {
    program_data: Vec<String>,
}

impl Instructions {
    /// Given an open file, consume the pointer to the file to construct an array of Hack instructions. Will result in error if
    /// obvious errors are found. This will only handle stripping out blanks and comments for now.
    pub fn import_from_file(file: File) -> Self {
        let reader = BufReader::new(file);
        let program_data = reader.lines()
            .filter(|line| {
                if let Ok(line_str) = line {
                    if let Some(prefix) = line_str.get(0..2) {
                        if prefix == "//" {
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .map(|x| {
                match x {
                    Ok(s) => s,
                    Err(_) => "".to_string(),
                }
            })
            .filter(|x| {
                x != ""
            })
            .collect();

        // At this point, the array should not contain any comments or blank lines.
        Self {
            program_data,
        }
    }

    fn parse_and_remove_labels(&mut self) -> Result<Vec<(String, isize)>, String> {
        let mut filtered_prog_data = Vec::<String>::new();
        let mut symbol_table_entries = Vec::<(String, isize)>::new();
        let mut counter = 0;
        let label_syms = &['(', ')'];
        for line in self.program_data.iter() {
            if let Some(c) = line.chars().nth(0) {
                if c == '(' {
                    let name = line.trim_matches(label_syms);
                    if name.chars().all(char::is_numeric) {
                        return Err(consts::LABEL_ERR.to_string());
                    } else {
                        symbol_table_entries.push((name.to_string(), counter + 1));
                    }
                } else {
                    filtered_prog_data.push(line.to_string());
                    counter += 1;
                }
            }
            // Otherwise, it's an empty string and we really don't need to do anything with this line.
        }
        self.program_data = filtered_prog_data;
        Ok(symbol_table_entries)
    }

    /// Call into the Instruction struct to handle getting the 
    pub fn process_one_instruction(&self, line: usize, sym_tbl: &mut SymbolTable) -> Result<String, String> {
        let line_text = self.program_data.get(line).ok_or("No valid instruction exists.".to_string())?;
        process_one_line(line_text, sym_tbl)
    }

    /// Simply returns how many lines there are.
    pub fn len(&self) -> usize {
        self.program_data.len()
    }

    pub fn get(&self, line_num: usize) -> Option<String> {
        self.program_data.get(line_num).cloned()
    }
}


// Misc Functions to help program

/// Simply prints instructions.
fn instructions() {
    println!("{}", consts::ASSEMBLER_HELP);
}

/// Handy function to remove the last section of a delimited string.
fn remove_last_delimiter(s: &str, delim: char) -> String {
    match s.rfind(delim) {
        Some(index) => s[0..index].to_string(),
        None => s.to_string(),
    }
}

/// Core API function to translate one line with the help of a symbol table.
pub fn process_one_line(line: &str, sym_tbl: &mut SymbolTable) -> Result<String, String> {
    let first_char = line.chars().nth(0).ok_or(consts::ASSEMBLER_EMPTY_LINE.to_string())?;
    if first_char == '@' {
        if let Ok(val) = line[1..].parse::<usize>() {
            return Ok(translate_addr_instr(val));
        }
        // else, this could be something we need to translate.
        let sub_str = &line[1..];
        let sym_str = sym_tbl.get(sub_str);
        match sym_str {
            Some(val) => {
                let pos_val = val as usize;
                return Ok(translate_addr_instr(pos_val));
            }

            _ => {
                sym_tbl.add_to_symbol_table_auto(sub_str.to_string());
                return Ok(translate_addr_instr(sym_tbl.get(sub_str).unwrap() as usize));
            }
        }
    }

    // Otherwise, this should be interpreted as a C instruction.
    translate_c_instr(line)
}

/// API function used to take in a parsed unsigned integer representing a memory location and format it as an A instruction for the Hack Assembler.
fn translate_addr_instr(parsed_val: usize) -> String {
    let temp = format!("{parsed_val:015b}");
    let mut out = "0".to_string();
    out.push_str(&temp);
    out
}  

/// API function used to handle C instructions from a text file.
fn translate_c_instr(code: &str) -> Result<String, String> {
    let mut dest: Option<&str> = None;
    let mut comp = code;
    let mut jmp: Option<&str> = None;
    let mut cnt_no_comp = 0;
    if comp.contains('=') {
        dest = comp.split('=').nth(0);
        comp = comp.split('=').nth(1).ok_or("Assignment operator used but nothing is assigned.".to_string())?;
    } else {
        cnt_no_comp += 1;
    }

    if comp.contains(';') {
        jmp = comp.split(';').nth(1);
        comp = comp.split(';').nth(0).ok_or("Comparison value not provided for jump operator.".to_string())?;
    } else {
        cnt_no_comp += 1;
    }

    if cnt_no_comp == 2 {
        return Err("No valid dest or jmp field defined, this is a malformed command.".to_string());
    }

    // At this point, we have the 3 components of an C-instruction line.
    let dest_component: String;
    // For assigning to memory
    if let Some(d) = dest {
        dest_component = build_bin_dest_string("ADM", d);
    } else {
        dest_component = "000".to_string();
    }

    // Jump, this is just a massive switch statement.
    let jmp_component: String;
    if let Some(j) = jmp {
        jmp_component = build_jmp_string(j)?;
    } else {
        jmp_component = "000".to_string();
    }

    // Now the section for comp
    let comp_component = build_comp_string(comp)?;
    let mut final_result = "111".to_string();
    final_result.push_str(&comp_component);
    final_result.push_str(&dest_component);
    final_result.push_str(&jmp_component);
    Ok(final_result)
}

/// Convenience function to handle the cases for the jump commands.
fn build_jmp_string(jmp_str: &str) -> Result<String, String> {
    let jmp_component: String;
    let j_upper = jmp_str.to_ascii_uppercase();
    let jmp_temp = match j_upper.as_str() {
        "JGT" => "001",
        "JEQ" => "010",
        "JGE" => "011",
        "JLT" => "100",
        "JNE" => "101",
        "JLE" => "110",
        "JMP" => "111",
        _ => "",
    };
    if jmp_temp == "" {
        return Err(format!("Unrecognized jump command \"{}\"", jmp_temp));
    }
    jmp_component = jmp_temp.to_string();
    Ok(jmp_component)
}

/// Organized section to handle the raw comp part of the string, provided that the beginning and end parts are out. Returns a [`Result`] with
/// the [Ok] branch being the processed string and the [Err] branch being an error message.
fn build_comp_string(raw_command: &str) -> Result<String, String> {
    let cmd_str = raw_command.to_ascii_uppercase();

    // The a bit determines whether or not this command involves M.
    let a_bit: char;
    if cmd_str.contains('M') {
        a_bit = '1';
    } else {
        a_bit = '0';
    }

    // From now on, we can treat any 'A' and 'M' characters the same, translate all into A. We can safely ignore the 'M' cases.
    let translated_cmd = cmd_str.replace("M","A"); // Use translated_cmd for all subsequent processing.

    let bin_repr = match translated_cmd.as_str() {
        "0" => "101010",
        "1" => "111111",
        "-1" => "111010",
        "D" => "001100",
        "A" => "110000",
        "!D" => "001101",
        "!A" => "110001",
        "-D" => "001111",
        "-A" => "110011",
        "D+1" | "1+D" => "011111",
        "A+1" | "1+A" => "110111",
        "D-1" => "001110",
        "A-1" => "110010",
        "D+A" | "A+D" => "000010",
        "D-A" => "010011",
        "A-D" => "000111",
        "D&A" | "A&D" => "000000",
        "D|A" | "A|D" => "010101",
        _ => "",
    };

    if bin_repr.len() != 6 {
        return Err(format!("Unrecognized comp command \"{}\"", translated_cmd));
    }

    let mut final_result = a_bit.to_string();
    final_result.push_str(bin_repr);
    Ok(final_result)
}

/// Simplified API function to handle the left hand assignment side of a C instruction.
fn build_bin_dest_string(chars_in_order: &str, input_str: &str) -> String {
    let mut out_str = String::new();
    let input_upper = input_str.to_ascii_uppercase();
    for c in chars_in_order.chars() {
        if input_upper.contains(c) {
            out_str.push('1');
        } else {
            out_str.push('0');
        }
    }
    out_str
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_a_instr() {
        // See if we can translate raw address decimal values correctly.
        assert_eq!(translate_addr_instr(17), "0000000000010001".to_string());
    }

    #[test]
    fn test_dest_string() {
        let chars_in_order = "ADM";
        let only_a = "A";
        let ad = "DA";
        assert_eq!(build_bin_dest_string(chars_in_order, only_a), "100".to_string());
        assert_eq!(build_bin_dest_string(chars_in_order, ad), "110".to_string());
    }
}