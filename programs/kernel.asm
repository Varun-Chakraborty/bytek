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
.ascii "Hello World\n\0"

.include "stdlib.asm"

START:
    MOVER R0, #MESSAGE
    CALL PRINT_STRING
