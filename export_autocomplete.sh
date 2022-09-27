cargo run -- --generate=zsh
cp _cmd /usr/local/share/zsh/site-functions/
autoload -Uz compinit
compinit