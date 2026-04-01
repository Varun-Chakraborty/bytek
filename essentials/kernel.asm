; pub fn kernel(&mut self) -> Result<(), VMError> {
;         ln!("Welcome to the kernel!\nType 'help' for a list of commands.");
;         let mut buffer = String::new();
;         let mut programloaded = false;
;         loop {
;             !("> ");
;             io::stdout().flush()?;
;             io::stdin().readline(&mut buffer)?;
;             match buffer.trim().tolowercase().asstr() {
;                 "exit" => {
;                     ln!("Goodbye!");
;                     return Ok(())
;                 },
;                 "run" => {
;                     if !programloaded {
;                         ln!("Please load a program first!");
;                         buffer.clear();
;                         continue;
;                     }
;                     ln!("Running the program");
;                     match self.run() {
;                         Ok() => {}
;                         Err(err) => {
;                             ln!("{}", err);
;                         }
;                     }
;                 },
;                 "debug on" => self.debug = true,
;                 "debug off" => self.debug = false,
;                 "load" => {
;                     !("Enter path to binary: ");
;                     io::stdout().flush()?;
;                     buffer.clear();
;                     io::stdin().readline(&mut buffer)?;
;                     let path = buffer.trim().tostring();
                    
;                     // test if the path is valid .bin file
;                     if !path.endswith(".bin") {
;                         ln!("Invalid path: {}", path);
;                         buffer.clear();
;                         continue;
;                     }
                    
;                     // test if file exists
;                     if !std::path::Path::new(&path).exists() {
;                         ln!("File does not exist: {}", path);
;                         buffer.clear();
;                         continue;
;                     }

;                     // test if file is a file
;                     match std::fs::metadata(&path) {
;                         Ok(metadata) if metadata.isfile() => {
;                             ln!("Loading program from: {}", path);
;                             match self.loadbinary(std::fs::read(path)?) {
;                                 Ok() => {
;                                     programloaded = true;
;                                 }
;                                 Err(err) => {
;                                     ln!("{}", err);
;                                 }
;                             }
;                         }
;                          => {
;                             ln!("Invalid path: {}", path);
;                             buffer.clear();
;                             continue;
;                         }
;                     }
;                 }
;                 "help" => ln!("Available commands:\n* exit\n* run\n* debug on\n* debug off\n* load\n* help"),
;                  => ln!("Unknown command: {}", buffer.trim()),
;             }
;             buffer.clear();
;         }
;     }




; Welcome message
JMP START

MESSAGE:
;WELCOME
DB 87
DB 101
DB 108
DB 99
DB 111
DB 109
DB 101

;SPACE
DB 32

;TO
DB 116
DB 111

;SPACE
DB 32

;THE
DB 116
DB 104
DB 101

;SPACE
DB 32

;KERNEL
DB 107
DB 101
DB 114
DB 110
DB 101
DB 108

;EXCLAMATION
DB 33

;NEWLINE
DB 10

;TYPE
DB 84
DB 121
DB 112
DB 101

;SPACE
DB 32

;SINGLEQUOTE
DB 39

;HELP
DB 104
DB 101
DB 108
DB 112

;SINGLEQUOTE
DB 39

;SPACE
DB 32

;FOR
DB 102
DB 111
DB 114

;SPACE
DB 32

;A
DB 97

;SPACE
DB 32

;LIST
DB 108
DB 105
DB 115
DB 116

;SPACE
DB 32

;OF
DB 111
DB 102

;SPACE
DB 32

;COMMANDS
DB 99
DB 111
DB 109
DB 109
DB 97
DB 110
DB 100
DB 115

;FULLSTOP
DB 46

;NEWLINE
DB 10

START:
    MOVEM R0, MESSAGE
