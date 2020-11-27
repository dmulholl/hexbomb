extern crate arguably;


use arguably::ArgParser;
use std::io::Seek;


const HELP: &str = "
Usage: hexbomb [FLAGS] [OPTIONS] [ARGUMENTS]

  A hex dump utility with style.

  The --offset option specifies the byte offset at which to begin reading.
  You can supply a positive or negative integer value for this option. A
  positive offset seeks forwards from the beginning of the file, a negative
  offset seeks backwards from the end of the file.

  For example, the following command will skip the first 128 bytes of the
  file:

    $ hexbomb <filename> --offset 128

  And the following command will display only the final 128 bytes of the
  file:

    $ hexbomb <filename> --offset -128

  Note that the --offset option cannot be used when piping or redirecting to
  stdin.

Arguments:
  [file]                    File to read. Defaults to reading from stdin.

Options:
  -l, --line <int>          Bytes per line in output (default: 16).
  -n, --number <int>        Number of bytes to read (default: all).
  -o, --offset <int>        Byte offset at which to begin reading.

Flags:
  -h, --help                Display this help text and exit.
  -v, --version             Display the version number and exit.
";


fn main() {
    let mut parser = ArgParser::new()
        .helptext(HELP)
        .version(env!("CARGO_PKG_VERSION"))
        .option("line l")
        .option("number n")
        .option("offset o");

    // Parse the command line arguments and convert string arguments to integers.
    if let Err(err) = parser.parse() {
        err.exit();
    }
    let (num_per_line, num_to_read, offset) = args_to_ints(&parser);
    let read_all = !parser.found("number");

    // Default to reading from stdin if no filename has been specified.
    if parser.args.len() == 0 {
        if offset != 0 {
            eprintln!("Error: STDIN does not support seeking to an offset.");
            std::process::exit(1);
        }
        let file = std::io::stdin();
        dump_file(file, read_all, num_to_read, num_per_line, 0);
        return;
    }

    // We only reach this point if a filename has been specified.
    let filepath = std::path::Path::new(&parser.args[0]);
    let mut file = match std::fs::File::open(&filepath) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Error: cannot open the specified file.");
            std::process::exit(1);
        }
    };

    // The display offset determines the first line number in the output.
    let mut display_offset: usize = 0;

    // A positive offset seeks forward from the beginning of the file.
    if offset > 0 {
        match file.seek(std::io::SeekFrom::Start(offset as u64)) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Error: cannot seek to the specified offset.");
                std::process::exit(1);
            }
        };
        display_offset = offset as usize;
    }

    // A negative offset seeks backwards from the end of the file.
    if offset < 0 {
        match file.seek(std::io::SeekFrom::End(0)) {
            Ok(file_size) => {
                display_offset = (file_size as i64 + offset) as usize;
            },
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        };
        match file.seek(std::io::SeekFrom::End(offset)) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Error: cannot seek to the specified offset.");
                std::process::exit(1);
            }
        };
    }

    dump_file(file, read_all, num_to_read, num_per_line, display_offset);
}



fn dump_file<T: std::io::Read>(
    mut file: T,
    read_all: bool,
    num_to_read: usize,
    num_per_line: usize,
    display_offset: usize
) {
    // Number of bytes remaining to be read, if we're reading a fixed number.
    let mut remaining = if read_all { usize::MAX } else { num_to_read };

    // We need to keep track of the offset for printing line numbers.
    let mut offset = display_offset;

    // Buffer for storing file input.
    let mut buffer: Vec<u8> = vec![0; num_per_line];

    print_top(num_per_line);

    loop {
        // Determine the maximum number of bytes to read this iteration.
        let max_bytes = if read_all {
            num_per_line
        } else if num_per_line < remaining {
            num_per_line
        } else {
            remaining
        };

        // Attempt to read up to max_bytes from the file.
        match file.read(&mut buffer[0..max_bytes]) {
            Ok(num_bytes) => {
                if num_bytes > 0 {
                    print_line(&buffer, num_bytes, offset, num_per_line);
                    offset += num_bytes;
                    remaining -= num_bytes;
                } else {
                    break;
                }
            },
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
    }

    print_bottom(num_per_line);
}


fn print_top(num_per_line: usize) {
    print!("┌──────────┬");

    for i in 0..num_per_line {
        if i > 0 && i % 8 == 0 {
            print!("──");
        }
        print!("───");
    }

    print!("─┬─");

    for i in 0..num_per_line {
        if i > 0 && i % 8 == 0 {
            print!("─");
        }
        print!("─");
    }

    println!("─┐");
}

fn print_bottom(num_per_line: usize) {
    print!("└──────────┴");

    for i in 0..num_per_line {
        if i > 0 && i % 8 == 0 {
            print!("──");
        }
        print!("───");
    }

    print!("─┴─");

    for i in 0..num_per_line {
        if i > 0 && i % 8 == 0 {
            print!("─");
        }
        print!("─");
    }

    println!("─┘");
}

fn print_line(bytes: &[u8], num_bytes: usize, offset: usize, num_per_line: usize) {

    // Write the line number.
    // print!("│ {:8X} │", offset);
    print!("│ {:width$X} │", offset, width = 8);

    for i in 0..num_per_line {
        if i > 0 && i % 8 == 0 {
            print!(" ┆");
        }
        if i < num_bytes {
            print!(" {:02X}", bytes[i]);
        } else {
            print!("   ");
        }
    }

    print!(" │ ");

    // Write a character for each byte in the printable ascii range.
    for i in 0..num_per_line {
        if i > 0 && i % 8 == 0 {
            print!("┆");
        }
        if i < num_bytes {
            if bytes[i] > 31 && bytes[i] < 127 {
                print!("{}", bytes[i] as char);
            } else {
                print!("·");
            }
        } else {
            print!(" ");
        }
    }

    println!(" │");

}




fn args_to_ints(parser: &ArgParser) -> (usize, usize, i64) {
    let num_per_line = match parser.value("line") {
        Some(str_val) => {
            match str_val.parse::<usize>() {
                Ok(int_val) => int_val,
                Err(_) => {
                    eprintln!("Error: cannot parse '{}' as a positive integer.", str_val);
                    std::process::exit(1);
                }
            }
        },
        None => 16
    };
    let num_to_read = match parser.value("number") {
        Some(str_val) => {
            match str_val.parse::<usize>() {
                Ok(int_val) => int_val,
                Err(_) => {
                    eprintln!("Error: cannot parse '{}' as a positive integer.", str_val);
                    std::process::exit(1);
                }
            }
        },
        None => 0
    };
    let offset = match parser.value("offset") {
        Some(str_val) => {
            match str_val.parse::<i64>() {
                Ok(int_val) => int_val,
                Err(_) => {
                    eprintln!("Error: cannot parse '{}' as an integer.", str_val);
                    std::process::exit(1);
                }
            }
        },
        None => 0
    };
    return (num_per_line, num_to_read, offset);
}


