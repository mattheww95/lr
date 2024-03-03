//! Simple implementation of the core utility ls in rust
//!
//! This is by no means a replacement for ls, but it is an initial way to help learn rust better.
//! I do not think anyone should use this over ls, however I do plan on adding some useful utility e.g. listing files in an FTP site
//!
use std::fs::DirEntry;
use chrono::NaiveDateTime;
use std::fs::{FileType};
use nix::unistd::{Group, Gid, User, Uid};
use colored::Colorize;
use std::os::unix::fs::MetadataExt;
use std::fs;
use std::path::Path;
use std::os::unix::fs::{PermissionsExt, FileTypeExt};
use clap::{Parser, ArgAction};


const KIBIBYTE: u128 = 1024;
const MEBIBYTE: u128 = u128::pow(KIBIBYTE, 2);
const GIBIBYTE: u128 = u128::pow(KIBIBYTE, 3);
const TEBIBYTE: u128 = u128::pow(KIBIBYTE, 4);
const PEBIBYTE: u128 = u128::pow(KIBIBYTE, 5);
const EXBIBYTE: u128 = u128::pow(KIBIBYTE, 6);
const ZEBIBYTE: u128 = u128::pow(KIBIBYTE, 7);
const YOBIBYTE: u128 = u128::pow(KIBIBYTE, 8);



#[derive(Parser)]
#[clap(disable_help_flag=true)]
#[command(version, about, long_about = None)]
struct Cli {

    /// List of directories
    #[clap(value_parser, num_args = 1.., value_delimiter=' ', default_value="./")]
    files: Vec<String>,

    /// Colour files/folders by type, specify to disable colouring
    #[clap(long, short, action=ArgAction::SetFalse, default_value_t=true)]
    colourize: bool,
    
    /// Show file sizes in a human readable format
    #[clap(long, short, action=ArgAction::SetTrue)]
    human: bool,

    /// Print long form output
    #[clap(long, short, action=ArgAction::SetTrue)]
    long: bool,


    /// Print all items in directory
    #[clap(long, short, action=ArgAction::SetTrue)]
    all: bool,


    /// Print help message
    #[clap(long, action=ArgAction::HelpLong)]
    help: Option<bool>,
}


/// Different device types represented e.g. sockets, pipes files etc are denoted here
enum DeviceType{
    Dir,
    BlockDevice,
    CharDevice,
    Symlink,
    Socket,
    Fifo,
    File,
}


/// Each DirectoryItem is represted here using different parts sliced from the DirEntry struct
struct DirectoryItem <'a>{
    /// path_abs: Absolute path to file
    /// path_disp: They way the path is displayed e.g. passed to the program
    /// file_name: The basename of the file or directory
    /// file_type: The DeviceType field listed in the Enum
    /// time: creation, modification or access time
    /// nlink: number of hard links connected to the item
    /// mode: File permissions number in u32
    /// group: user group
    /// size: file size
    /// user: user created the file
    /// executable: if file is executable
    /// defaults: Defaults value passed in.
    path_abs: String,
    path_disp: String,
    file_name: String,
    file_type: DeviceType,
    time: i64,
    nlink: u64,
    mode: u32,
    group: Group,
    size: u128,
    user: User,
    executable: bool,
    defaults: &'a Defaults,
}


/// Defaults arguments passed in
struct Defaults {
    /// colourize: Denote if displayed values should be colourized
    /// human_readable: Marking if size values should be displayed in a human readable format
    /// long_form: list directories in long form
    /// all: Bool denoting if full path value should be displayed
    colourize: bool,
    human_readable: bool,
    long_form: bool,
    all: bool,
}


impl DirectoryItem<'_>  {
    fn from_file<'a>(path: &Path, defaults: &'a Defaults) -> DirectoryItem<'a> {
        let path_buf= path;
        let metadata = fs::metadata(path).unwrap();
        let file_type = DirectoryItem::file_type(metadata.file_type());
        let mode = metadata.permissions().mode();
        let mut executable = false;
        if mode & 0o001 == 0o1 {
            executable = true;
        }
        let group = Group::from_gid(Gid::from_raw(metadata.gid())).unwrap().unwrap();
        let user = User::from_uid(Uid::from_raw(metadata.uid())).unwrap().unwrap();
        let nlink = metadata.nlink();
        let time = metadata.ctime();
        let size = metadata.size();
        DirectoryItem{
            file_type: file_type,
            file_name: path_buf.file_name().unwrap().to_str().unwrap().to_string(),
            time: time,
            nlink: nlink,
            mode: mode,
            group: group,
            size: size as u128,
            user:user,
            executable: executable,
            path_abs: fs::canonicalize(path).unwrap().display().to_string(),
            path_disp: path_buf.display().to_string(),
            defaults: defaults,
        }

    }

    fn from_dir_entry<'a>(path: DirEntry, defaults: &'a Defaults) -> DirectoryItem<'a> {

        let path_buf= path.path();
        let metadata = path.metadata().unwrap();
        let file_type = DirectoryItem::file_type(metadata.file_type());
        let mode = metadata.permissions().mode();
        let mut executable = false;
        if mode & 0o001 == 0o1 {
            executable = true;
        }
        let group = Group::from_gid(Gid::from_raw(metadata.gid())).unwrap().unwrap();
        let user = User::from_uid(Uid::from_raw(metadata.uid())).unwrap().unwrap();
        let nlink = metadata.nlink();
        let time = metadata.ctime();
        let size = metadata.size();
        DirectoryItem{
            file_type: file_type,
            file_name: path_buf.file_name().unwrap().to_str().unwrap().to_string(),
            time: time,
            nlink: nlink,
            mode: mode,
            group: group,
            size: size as u128,
            user:user,
            executable: executable,
            path_abs: fs::canonicalize(path_buf.clone()).unwrap().display().to_string(),
            path_disp: path_buf.clone().display().to_string(),
            defaults: defaults,
        }

    }

    fn time(&self) -> String {
        let d = NaiveDateTime::from_timestamp_opt(self.time, 0);

        let time_stamp_str = d.unwrap().format("%Y %b %d %H:%M").to_string();
        return time_stamp_str;
    }

    fn file_path(&self) -> String {
        let mut display = &self.file_name;
        if self.defaults.long_form {
            display = &self.path_abs;
        }

        if self.defaults.colourize {
            return self.coloured_paths(&display);
        }
        return self.normal_paths(&display);

    }

    fn coloured_paths(&self, display: &str) -> String {
        let out_str = match self.file_type {
            DeviceType::Symlink => if self.defaults.long_form {
                format!("{} -> {}", self.path_disp.bright_cyan(), self.path_abs)
            } else {  
                format!("{}", self.path_disp.bright_cyan())
            },
            DeviceType::BlockDevice => format!("{}", display.bright_yellow()),
            DeviceType::CharDevice => format!("{}", display.bright_magenta()),
            DeviceType::Fifo => format!("{}", display),
            DeviceType::Socket => format!("{}", display),
            DeviceType::Dir => format!("{}", display.bright_blue()),
            _ => if self.executable { 
                        format!("{}", display.bright_green())
                    } else {
                        format!("{}", display)
                    }
            };
        return out_str;

    }

    fn normal_paths(&self, display: &str) -> String {
        let out_str = match self.file_type {
            DeviceType::Symlink => if self.defaults.long_form {
                format!("{} -> {}", self.path_disp, self.path_abs)
            } else {  
                format!("{}", self.path_disp)
            },
            DeviceType::BlockDevice => format!("{}", display),
            DeviceType::CharDevice => format!("{}", display),
            DeviceType::Fifo => format!("{}", display),
            DeviceType::Socket => format!("{}", display),
            DeviceType::Dir => format!("{}", display),
            _ => format!("{}", display)
            };
            return out_str;
    }

    fn file_type(f_type: FileType) -> DeviceType {
        let ftype = match f_type {
            x if x.is_socket() => DeviceType::Socket,
            x if x.is_symlink() => DeviceType::Symlink,
            x if x.is_char_device() => DeviceType::CharDevice,
            x if x.is_fifo() => DeviceType::Fifo,
            x if x.is_block_device() => DeviceType::BlockDevice,
            x if x.is_dir() => DeviceType::Dir,
            _ => DeviceType::File,
        };
        ftype
    }

    fn permission_char(&self) -> char {

        match self.file_type {
            DeviceType::Dir => 'd',
            DeviceType::BlockDevice => 'b',
            DeviceType::Symlink => 'l',
            DeviceType::CharDevice => 'c',
            DeviceType::Socket => 's',
            DeviceType::Fifo => 'p',
            DeviceType::File => '-',

        }
    }

    fn convert_units(size: u128) -> String {
        let magnitude = match size as u128 {
            e if e < KIBIBYTE => format!("{}B", e),
            e if e < MEBIBYTE => format!("{}KiB", e / KIBIBYTE),
            e if e < GIBIBYTE => format!("{}MiB", e / MEBIBYTE),
            e if e < TEBIBYTE => format!("{}TiB", e / GIBIBYTE),
            e if e < PEBIBYTE => format!("{}PiB", e / TEBIBYTE),
            e if e < EXBIBYTE => format!("{}EiB", e / PEBIBYTE),
            e if e < ZEBIBYTE => format!("{}ZiB", e / EXBIBYTE),
            e if e < YOBIBYTE => format!("{}YiB", e / ZEBIBYTE),
            e @ _ => format!("{}YiB wtf... how?", e / YOBIBYTE),
        };
        return magnitude
    }


    fn size(&self) -> String {
        // Return the size as a string
        if self.defaults.human_readable {
            return DirectoryItem::convert_units(self.size);
        }
        return self.size.to_string();
    }


    fn permissions_string(&self) -> String {
        let system  = self.mode & 0o700;
        let group = self.mode & 0o070;
        let user_  = self.mode & 0o007; 
    
        format!("{}{}{}{}", self.permission_char(),
            DirectoryItem::permissions_triplet(system),
            DirectoryItem::permissions_triplet(group),
            DirectoryItem::permissions_triplet(user_))
    }

    fn permissions_triplet(value:u32) -> &'static str {
        match value {
            0 => "---",
            0o1|0o10|0o100 => "--x",
            0o2|0o20|0o200 => "-w-",
            0o3|0o30|0o300 => "-wx",
            0o4|0o40|0o400 => "r--",
            0o5|0o50|0o500 => "r-x",
            0o6|0o60|0o600 => "rw-",
            _ => "rwx"
        }
    }

    fn print_long(&self) {
        println!("{} {} {} {} {} {} {}", self.permissions_string(), self.nlink, self.group.name, self.user.name, self.size(), self.time(), self.file_path());
    }
}


fn list_contents(dir: &Path, defaults: &Defaults) -> () {
    if dir.is_dir() {
        let paths = fs::read_dir(dir).unwrap();

        for path in paths {

            let data = path.unwrap();
            if !defaults.all && data.file_name().to_str().unwrap().starts_with(".") {
                continue;
            }
            let new_value = DirectoryItem::from_dir_entry(data, defaults);
            if defaults.long_form{
                new_value.print_long()
            }else {
                print!("{} ", new_value.file_path());
            }
        }
        if !defaults.long_form {
            println!();
        }
        return ();
    }
    let file = DirectoryItem::from_file(dir, defaults);
    if defaults.long_form {
        file.print_long();
    }else {
        println!("{}", file.file_path());
    }
    return ();
}

fn main(){

    let args = Cli::parse();
    let defaults = Defaults{
        colourize: args.colourize, 
        human_readable: args.human, 
        long_form: args.long,
        all: args.all};

    for path in args.files.iter(){
        list_contents(path.as_ref(), &defaults);
    }
    
}



#[test]
fn convert_units(){
    assert_eq!("4B", DirectoryItem::convert_units(4));
    assert_eq!("1KiB", DirectoryItem::convert_units(KIBIBYTE));
    assert_eq!("1MiB", DirectoryItem::convert_units(MEBIBYTE));
    assert_eq!("1TiB", DirectoryItem::convert_units(GIBIBYTE));
    assert_eq!("1PiB", DirectoryItem::convert_units(TEBIBYTE));
    assert_eq!("1EiB", DirectoryItem::convert_units(PEBIBYTE));
    assert_eq!("1ZiB", DirectoryItem::convert_units(EXBIBYTE));
    assert_eq!("1YiB", DirectoryItem::convert_units(ZEBIBYTE));
    assert_eq!("1YiB wtf... how?", DirectoryItem::convert_units(YOBIBYTE));
}


#[test]
fn permissions_triplet(){
    assert_eq!("---", DirectoryItem::permissions_triplet(0));
    assert_eq!("--x", DirectoryItem::permissions_triplet(1));
    assert_eq!("-w-", DirectoryItem::permissions_triplet(2));
    assert_eq!("-wx", DirectoryItem::permissions_triplet(3));
    assert_eq!("r--", DirectoryItem::permissions_triplet(4));
    assert_eq!("r-x", DirectoryItem::permissions_triplet(5));
    assert_eq!("rw-", DirectoryItem::permissions_triplet(6));
    assert_eq!("rwx", DirectoryItem::permissions_triplet(7));
}