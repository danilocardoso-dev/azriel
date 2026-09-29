# AZRIEL — Histórico de consolidação dos módulos

Data: **25/09/2026**

## Objetivo

Reduzir a navegação principal aos módulos que permanecem úteis, congelar o Engineering View e reorganizar as informações operacionais no Command Center sem apagar dados históricos do banco local.

## Estado final dos módulos

### Engineering View

Status: **congelado**.

- removido do menu e do roteamento principal;
- não é importado nem inicializado na abertura do Azriel;
- suas tools deixaram de ser registradas no AI Core;
- o código-fonte e as tabelas existentes foram preservados como referência histórica e para uma eventual retomada controlada;
- o build de produção não gera mais um chunk do Engineering View.

### Mapa Stark

Status: **substituído por Estudos**.

- a entrada `Mapa Stark` foi removida;
- a nova entrada `Estudos` contém somente a experiência de roadmaps;
- foram removidas da interface as telas de visão geral, conhecimento, pesquisa, evolução e lacunas;
- ferramentas de IA específicas do antigo Mapa Stark e do Learning Engine deixaram de ser registradas;
- roadmaps, etapas, tópicos, atividades, progresso, criação, edição e exclusão continuam disponíveis;
- referências históricas necessárias ao editor de roadmap permanecem somente como dados de compatibilidade, sem telas próprias.

### Formação

Status: **removido**.

- removida do menu, do roteamento e do carregamento inicial;
- editor, página, serviço e repositório frontend foram excluídos;
- ferramentas de Formação deixaram de ser expostas pelo AI Core;
- tabelas persistidas não foram apagadas para evitar perda irreversível de dados históricos.

### Sistema

Status: **removido como módulo independente**.

- página e entrada de navegação removidas;
- telemetria reutilizada no Command Center;
- gerenciamento de workspaces movido para `Automação > Workspaces`;
- configurações gerais e do AI Core continuam em `Configurações`.

## Command Center

O Command Center foi reconstruído como painel operacional compacto e agora reúne:

- estado do Azriel Core;
- CPU, memória, armazenamento e tempo ativo do dispositivo;
- roadmap ativo e progresso de Estudos;
- operações diárias;
- projetos;
- automação e rotinas;
- AI Core;
- Market Lab;
- atalhos para os módulos ativos.

Os painéis antigos baseados em topologia, lacunas e métricas do Mapa Stark foram removidos.

## Navegação resultante

```text
Command Center
AI Core
Operações Diárias
Projetos
Estudos
Market Lab
Automação
Configurações
```

## Compatibilidade e dados

Nenhuma migration destrutiva foi criada. Estruturas SQLite antigas de Engineering, Formação e conhecimento foram mantidas para preservar dados existentes e permitir auditoria ou recuperação futura. A remoção desta etapa é funcional e visual, não uma exclusão irreversível do histórico do usuário.

## Validação automatizada

- ESLint: aprovado;
- frontend: **154 testes aprovados em 23 arquivos**;
- TypeScript e build Vite: aprovados;
- build Vite transformou 112 módulos e não gerou chunk do Engineering View;
- Rust `cargo check --all-targets`: aprovado usando diretório de build isolado, pois uma instância do Azriel estava aberta e mantinha o lock do diretório `target` padrão;
- Rust: **206 testes aprovados e 1 smoke test manual do Ollama ignorado**;
- aviso Rust preexistente: visibilidade de `Candle` em `compute_features` no Market Lab.

## Validação operacional pendente

Após esta consolidação, o operador deve confirmar no aplicativo Tauri:

1. que o menu contém somente os oito módulos listados acima;
2. que `Estudos` abre diretamente os roadmaps;
3. que Engenharia, Formação, Mapa Stark e Sistema não podem ser abertos pela navegação;
4. que o Command Center exibe telemetria e tempo ativo;
5. que `Automação > Workspaces` preserva cadastro, edição e autorização de pastas;
6. que os módulos restantes continuam operando normalmente.
