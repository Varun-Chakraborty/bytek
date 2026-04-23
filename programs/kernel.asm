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

.align

MESSAGE:
;WELCOME
.byte 87
.byte 101
.byte 108
.byte 99
.byte 111
.byte 109
.byte 101

;SPACE
.byte 32

;TO
.byte 116
.byte 111

;SPACE
.byte 32

;THE
.byte 116
.byte 104
.byte 101

;SPACE
.byte 32

;KERNEL
.byte 107
.byte 101
.byte 114
.byte 110
.byte 101
.byte 108

;EXCLAMATION
.byte 33

;NEWLINE
.byte 10

;TYPE
.byte 84
.byte 121
.byte 112
.byte 101

;SPACE
.byte 32

;SINGLEQUOTE
.byte 39

;HELP
.byte 104
.byte 101
.byte 108
.byte 112

;SINGLEQUOTE
.byte 39

;SPACE
.byte 32

;FOR
.byte 102
.byte 111
.byte 114

;SPACE
.byte 32

;A
.byte 97

;SPACE
.byte 32

;LIST
.byte 108
.byte 105
.byte 115
.byte 116

;SPACE
.byte 32

;OF
.byte 111
.byte 102

;SPACE
.byte 32

;COMMANDS
.byte 99
.byte 111
.byte 109
.byte 109
.byte 97
.byte 110
.byte 100
.byte 115

;FULLSTOP
.byte 46

;NEWLINE
.byte 10

;\NULL
.byte 0

START:
    MOVER R0, #MESSAGE
LOOP:
    MOVER R1, [R0]
    JZ EXIT
    OUT_CHAR R1
    ADD R0, #1
    JMP LOOP
EXIT: HALT
