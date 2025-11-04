//
O rustanalyzer é uma extensão para programar em Rust capaz de inicializar um "servidor" onde testa suas alterações de forma prévia. Ele não é capaz de ver arquivos .env, muito menos as próprias variaveis do sistema, então acusa um erro.

Ao adicionar a mesma chave do .env no settings.json, ele passa a ler aquele arquivo como as variaveis de ambiente, então lembre-se de copiar suas chaves enquanto estiver em teste

Erros do analyzer sobre variáveis de ambiente não influenciam na compilação do projeto