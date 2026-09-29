# AZRIEL AI v0.1 — Model Lifecycle Control

Status: implementado em 26/09/2026. Validação operacional com o servidor Ollama via Tailscale pendente.

## Objetivo

O Command Center controla o carregamento do modelo configurado na memória do Ollama. O Azriel não inicia, encerra ou administra o servidor Ubuntu, o serviço Ollama, o Tailscale, Docker ou a GPU.

Desativar a IA significa enviar uma solicitação global de unload somente para o modelo configurado. O servidor Ollama permanece online e pode continuar atendendo outros modelos e clientes.

## Arquitetura reutilizada

O recurso estende o cliente HTTP já existente em `src-tauri/src/ollama.rs`, os comandos do AI Core e as configurações persistidas em `ai_settings`. Não existe segundo provider, daemon, SSH, scheduler ou novo módulo de interface.

Os comandos Tauri são:

- `get_local_ai_model_status`;
- `load_local_ai_model`;
- `unload_local_ai_model`.

Os comandos não recebem endpoint nem modelo do botão. O backend lê esses valores diretamente das configurações persistidas.

## API Ollama

O estado real vem de:

```text
GET {endpoint}/api/ps
```

O modelo é ativado com:

```json
{"model":"<modelo configurado>","keep_alive":-1,"stream":false}
```

O modelo é desativado com:

```json
{"model":"<modelo configurado>","keep_alive":0,"stream":false}
```

Após o POST, o backend consulta `/api/ps` até quatro vezes, com intervalo de 450 ms. A operação somente termina como sucesso quando o modelo correto aparece ou desaparece. HTTP 200 isolado não é considerado confirmação.

`/api/tags` é consultado apenas quando o modelo configurado não está em memória, permitindo diferenciar `UNLOADED` de `NOT_AVAILABLE` sem executar pull automático.

## Estados

- `ONLINE/OFFLINE/UNKNOWN`: disponibilidade do servidor;
- `UNLOADED`: servidor online, modelo instalado e fora da memória;
- `LOADING`: ativação em andamento;
- `LOADED`: o modelo configurado está presente em `/api/ps`;
- `UNLOADING`: desativação em andamento;
- `NOT_AVAILABLE`: modelo configurado não aparece nos modelos instalados;
- `ERROR`: resposta inválida, erro HTTP ou configuração inválida.

Outro modelo carregado nunca é confundido com o modelo configurado. A comparação é exata, permitindo apenas equivalência explícita com a tag `latest` quando uma das formas omite a tag.

## Concorrência e atualização

Uma trava atômica no backend impede duas operações de load/unload simultâneas. Conversas também aguardam o fim de uma transição de lifecycle, evitando que um chat reverta a operação em andamento.

O Command Center consulta o estado ao ser aberto e a cada 20 segundos enquanto permanece montado. Não há polling global, polling por segundo ou carregamento automático no startup.

## Política de keep_alive

Quando o operador ativa explicitamente o modelo pelo Command Center, o processo atual do Azriel registra essa intenção em memória. As conversas normais passam a enviar `keep_alive: -1`, inclusive chamadas reutilizadas pelo Study Lab e Market Lab.

Após uma desativação confirmada, essa marca é removida e as chamadas voltam à política normal do Ollama. Uma nova conversa pode carregar o modelo novamente porque precisa dele para responder, mas não o fixa indefinidamente sem nova ativação explícita.

A intenção de pin não é persistida no SQLite. Depois de reiniciar o Azriel, `/api/ps` continua refletindo corretamente o estado global, mas uma nova ativação explícita é necessária para restaurar a política permanente do cliente Azriel. Fechar o aplicativo nunca envia unload.

## Política de raciocínio

As chamadas de chat enviam `think: false`. O Azriel consome `message.content` e não utiliza o campo separado `message.thinking`; permitir raciocínio oculto no `qwen3.5:4b` consumia o limite de tokens e podia exceder o timeout antes de produzir conteúdo visível.

A desativação vale no cliente compartilhado pelo AI Core, Study Lab e respostas estruturadas do Market Lab. Ela não altera o modelo configurado, o histórico ou os limites de geração de cada perfil.

## Endpoint e segurança

Somente HTTP sem credenciais, query, fragmento ou subcaminho é aceito. Hosts permitidos:

- localhost e loopback;
- redes privadas e link-local;
- faixa Tailscale CGNAT `100.64.0.0/10`;
- nomes MagicDNS simples ou terminados em `.ts.net`.

IPs públicos, domínios públicos e HTTPS arbitrário continuam rejeitados. O recurso não armazena ou utiliza senha, root, sudo, chave privada ou comando remoto.

## Logging e erros

O log `[AI_MODEL]` contém somente ação, provider, modelo sanitizado, duração, resultado e categoria segura de erro. Payloads, mensagens, credenciais e respostas integrais não são registrados.

São tratados timeout, conexão recusada, erro HTTP, resposta malformada, modelo não instalado, concorrência e falha de confirmação.

## Teste operacional

1. Configure o endpoint Tailscale e o modelo no AI Core.
2. Abra o Command Center e confirme `OLLAMA SERVER ONLINE`.
3. Com o modelo fora da memória, confirme `UNLOADED` e clique `ATIVAR IA`.
4. Confirme a transição `LOADING` e depois `LOADED`.
5. Envie uma mensagem normal e confirme a resposta do modelo configurado.
6. Volte ao Command Center, clique `DESATIVAR IA` e confirme `UNLOADING` seguido de `UNLOADED`.
7. Confirme que o Ollama continua online.
8. Carregue o mesmo modelo por outro cliente e confirme que o próximo refresh do Command Center mostra `LOADED`.
9. Teste endpoint indisponível e confirme mensagem compreensível, sem stack trace.

## Limitações deliberadas

Não há auto-unload, scheduler, wake-on-demand, pull/delete de modelos, roteador multi-modelo, controle de GPU, SSH, systemctl, Docker ou desligamento do servidor. O estado de pin explícito do Azriel dura somente durante o processo atual.
