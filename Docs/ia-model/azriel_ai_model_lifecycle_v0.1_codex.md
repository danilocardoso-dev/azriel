# AZRIEL AI v0.1 — Model Lifecycle Control
## Command Center Integration — Prompt para Codex

## Contexto
O AZRIEL é cliente de uma LLM local executada em servidor Ubuntu via Ollama/Tailscale. O modelo atual é `qwen3.5:4b`. O Azriel não hospeda a LLM.

Esta feature pertence **exclusivamente ao Command Center existente**. NÃO criar novo módulo, rota principal, página ou entrada na sidebar.

Objetivo: controlar o carregamento do modelo na memória via API Ollama, sem SSH, daemon auxiliar, systemctl ou privilégios administrativos.

## 1. Semântica
Distinguir:
- servidor/Ollama OFFLINE;
- Ollama ONLINE + modelo UNLOADED;
- modelo LOADING;
- modelo LOADED;
- modelo UNLOADING;
- ERROR/UNKNOWN.

Desativar IA = descarregar o modelo da memória. Não desligar servidor/Ollama/Tailscale.

## 2. API Ollama
Reutilizar base URL/model configurados no AI Core.

Status:
`GET {base_url}/api/ps`

Modelo configurado presente => LOADED.
Ausente => UNLOADED.
Falha de API/conexão != UNLOADED.

Load:
`POST {base_url}/api/generate`
```json
{"model":"<configured_model>","keep_alive":-1,"stream":false}
```

Unload:
```json
{"model":"<configured_model>","keep_alive":0,"stream":false}
```

Não hardcode IP/model se já configuráveis.

## 3. Confirmação pós-ação
HTTP 200 não basta.

ATIVAR:
`POST keep_alive=-1 → GET /api/ps → confirmar modelo presente`.

DESATIVAR:
`POST keep_alive=0 → GET /api/ps → confirmar modelo ausente`.

Permitir poucas tentativas de confirmação com delay curto e limite rígido. Sem loop infinito/polling agressivo.

## 4. Inspeção obrigatória
Antes de codificar:
1. analisar AI Core/provider Ollama;
2. localizar config de URL/model;
3. HTTP client/timeouts/errors;
4. Command Center/cards/status;
5. state management;
6. logger;
7. registrar plano mínimo.

Repositório é fonte de verdade. Não criar segundo OllamaProvider/HTTP client, AIManager, ModelRouter, LLMGateway, RemoteServerManager ou SSHService sem necessidade real.

## 5. Backend
Estender provider/serviço existente. Responsabilidades conceituais:
- get_local_ai_status
- load_local_ai_model
- unload_local_ai_model

Adaptar naming ao projeto.

Frontend não chama Ollama diretamente se a arquitetura atual centraliza providers no backend.

Status DTO conceitual:
- provider
- model
- server_status
- model_status
- loaded
- expires_at nullable
- last_checked_at
- error nullable

Não retornar dados sensíveis.

## 6. Command Center UI
Adicionar card compacto, seguindo design existente:

```text
IA LOCAL
Qwen3.5 4B                 ● ONLINE
Ollama Server              ● ONLINE
Modelo                       LOADED
Provider                     OLLAMA

                    [ DESATIVAR IA ]
```

UNLOADED:
```text
Qwen3.5 4B                ○ STANDBY
Ollama Server             ● ONLINE
Modelo                     UNLOADED

                       [ ATIVAR IA ]
```

SERVER OFFLINE:
```text
Qwen3.5 4B                ○ OFFLINE
Ollama Server             ○ OFFLINE
Servidor indisponível

                        [ VERIFICAR ]
```

Adaptar visual ao Command Center real. Reutilizar cards, badges, dots, tipografia, spacing e botões existentes.

## 7. Estados dos botões
- UNLOADED => ATIVAR IA
- LOADED => DESATIVAR IA
- LOADING => ATIVANDO... desabilitado
- UNLOADING => DESATIVANDO... desabilitado
- SERVER OFFLINE => VERIFICAR

Bloquear duplo clique e ações concorrentes.

## 8. Status real
Ao abrir Command Center, consultar estado real. Não assumir último estado salvo.

Pode haver refresh manual e refresh periódico moderado somente enquanto a tela estiver visível (ex.: 15–30s, adaptado ao padrão existente). Nunca a cada segundo.

## 9. Modelo correto
Comparar `/api/ps` com o modelo configurado. Outro modelo carregado não significa Qwen carregado.

Quando possível, distinguir UNLOADED de NOT_AVAILABLE/modelo não instalado usando infraestrutura/endpoints existentes. Não fazer `pull` automaticamente.

## 10. Load / Unload
LOAD: validar config → LOADING → POST → confirmação `/api/ps` → LOADED ou erro.

UNLOAD: UNLOADING → POST → confirmação `/api/ps` → UNLOADED ou erro.

Operações assíncronas; UI não pode congelar.

## 11. Conversas normais / keep_alive
Revisar como requests normais usam `keep_alive`.

A intenção manual do usuário não deve ser revertida de forma inesperada. Se o modelo foi explicitamente ativado para permanecer carregado, usar política coerente com o AI Core atual e documentar a decisão.

Não implementar auto-unload/scheduler nesta versão.

## 12. Startup / shutdown do Azriel
Não carregar modelo automaticamente ao iniciar Azriel. Apenas consultar status quando necessário.

Fechar Azriel NÃO descarrega automaticamente o modelo: Ollama pode ser usado por VS Code/Continue ou outros clientes.

O estado é global no Ollama. Se outro cliente carregar o Qwen, o Command Center deve refletir LOADED.

Ao DESATIVAR, o usuário solicita unload global daquele modelo no Ollama. Documentar.

## 13. Segurança
Proibido:
- SSH;
- senha/root/sudo/private key;
- comandos remotos;
- systemctl;
- Docker control;
- exposição pública do Ollama.

Toda operação ocorre pela API Ollama acessível via Tailscale.

## 14. Erros
Tratar:
- server unreachable;
- timeout;
- connection refused;
- HTTP error;
- malformed response;
- model unavailable;
- load not confirmed;
- unload not confirmed.

Mostrar mensagem compreensível, sem stack trace.

## 15. Logging
Usar logger existente. Registrar action STATUS/LOAD/UNLOAD, provider, model, duration, result e error category. Não logar credenciais/dados sensíveis desnecessários.

## 16. Métricas opcionais
Se `/api/ps` fornecer `expires_at`, size/VRAM etc., podem ser exibidos discretamente quando úteis. Não tornar requisito e não inventar estimativas de RAM/CPU/energia.

## 17. Testes
Status:
- online + Qwen loaded
- online + Qwen absent
- outro modelo loaded
- lista vazia
- timeout/conexão inválida/malformed

Load:
- sucesso + confirmação
- POST falha
- confirmação falha
- timeout
- modelo indisponível
- duplo clique/transições

Unload:
- sucesso + confirmação
- POST falha
- ainda presente após retries
- servidor offline
- duplo clique

Regressão:
- chat existente
- Study Lab AI
- Market Lab
- provider config
- outros providers quando existirem
- Command Center

## 18. Teste integrado
```text
Command Center
→ IA LOCAL UNLOADED
→ ATIVAR IA
→ LOADING
→ /api/ps confirma LOADED
→ mensagem normal recebe resposta Qwen
→ DESATIVAR IA
→ UNLOADING
→ /api/ps confirma UNLOADED
→ Ollama continua ONLINE
```

## 19. Fora do escopo
Não implementar SSH, daemon auxiliar, systemctl, Docker/server shutdown, stop/start Ollama, auto-unload, scheduler, wake-on-demand, multi-model router, model pull/delete, GPU controls, novo módulo/sidebar/page, RAG ou agents.

## 20. Critérios de aceite
1. controle está no Command Center;
2. nenhuma nova sidebar/page/módulo;
3. AI Core/provider existente reutilizado;
4. URL/model configuráveis;
5. `/api/ps` é fonte real de status;
6. server/model status diferenciados;
7. modelo correto identificado;
8. ATIVAR usa keep_alive=-1;
9. DESATIVAR usa keep_alive=0;
10. Ollama permanece ativo;
11. load/unload confirmados por `/api/ps`;
12. retries limitados;
13. LOADING/UNLOADING e bloqueio concorrente funcionam;
14. offline/timeout/error tratados;
15. startup não carrega automaticamente;
16. fechar Azriel não descarrega automaticamente;
17. estado de outros clientes é refletido;
18. nenhum SSH/admin credential/hardcode IP;
19. nenhuma automação futura antecipada;
20. chat/Study Lab/Market Lab permanecem funcionais;
21. testes/build passam;
22. Tauri inicia;
23. documentação existe.

## 21. Documentação
Criar `docs/ai/model-lifecycle-v0.1.md` ou caminho equivalente ao padrão real. Documentar API usada, semântica de estados, load/unload, confirmação, keep_alive de conversas normais, comportamento com outros clientes, erros, segurança, testes e limitações.

## 22. Entrega Codex
Ao concluir, resumir:
- arquivos alterados/criados;
- arquitetura reutilizada;
- commands/API;
- mudanças no Command Center;
- política keep_alive;
- testes/resultados;
- build/Tauri;
- limitações.

## Resultado esperado
`COMMAND CENTER → IA LOCAL → STATUS REAL → ATIVAR/DESATIVAR → API OLLAMA → CONFIRMAÇÃO /api/ps`.

A implementação deve ser pequena, operacional e coerente com o AZRIEL atual. Não transformar controle de lifecycle em novo subsistema.
