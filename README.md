# Revenue OS para Windows

Aplicativo de computador em Rust/Tauri para acessar o Revenue OS com sua conta existente.

[Baixar para Windows 10/11 (64 bits)](https://github.com/andrefonn/revenueos-desktop/releases/latest/download/RevenueOS-Windows-x64-Setup.exe)

Instale e abra pelo atalho da área de trabalho ou pelo menu Iniciar. O programa requer internet para acessar o sistema. Usa o WebView2 do Windows; o instalador oferece sua instalação quando necessário.

## Atualizações

O aplicativo verifica novas versões ao abrir. Também é possível usar **Aplicativo → Verificar atualizações**. Você escolhe quando atualizar: o download e a verificação de assinatura ocorrem no aplicativo, que fecha para instalar e reabrir. Não é preciso baixar outra versão pelo navegador. Adie se houver trabalho em andamento.

## Segurança

Login, autorização e integrações ficam no servidor HTTPS do Revenue OS. O instalador não inclui chaves de Meta, WhatsApp, Asaas ou GitHub. Conteúdo remoto não recebe permissões para executar comandos nativos. As atualizações exigem uma assinatura correspondente à chave pública embutida. Essa assinatura é própria do atualizador e não constitui assinatura Authenticode; o Windows pode mostrar aviso na primeira instalação.

## Desenvolvimento e releases

Pré-requisitos: Windows, Rust stable >=1.90, ferramentas C++ MSVC e SDK Windows, Node 24 e WebView2.

```powershell
npm ci
npm test
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

O release é publicado pelo GitHub Actions quando uma tag `vX.Y.Z` é enviada. Atualize a versão em `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` e `package.json` antes da tag. O workflow recusa divergência entre tag e versão e exige `TAURI_SIGNING_PRIVATE_KEY` nos secrets do repositório. A chave privada nunca deve entrar no Git. A primeira chave pública acompanha o aplicativo e deve ser preservada em novas versões.

Os assets públicos são o instalador, sua assinatura, `latest.json` e `SHA256SUMS.txt`. O código do servidor não faz parte deste repositório.
