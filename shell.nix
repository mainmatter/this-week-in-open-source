{ sources ? import ./nix/sources.nix,
  pkgs ? import sources.nixpkgs {}
}: with pkgs;
 
let inherit (lib) optional optionals;
in  
 mkShell {
  buildInputs = [
   # libiconv, openssl, pkgconfig are needed for openssl dependent packages
   libiconv                                                                                                                                                        
   openssl
   pkg-config                         
   # Rust tooling                    
   cargo                             
   rustup                            
   rust-analyzer
   rustc
  ]; 
 } 
