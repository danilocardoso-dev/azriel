Você está trabalhando no projeto **Azriel**.

Antes de alterar qualquer arquivo:

1. leia o README completo;
2. inspecione toda a estrutura do repositório;
3. identifique o que já existe;
4. preserve código útil;
5. só então implemente a nova versão.

# Objetivo

Implementar:

**Azriel v0.4 — HUD Vivo**

Status: **✅ Concluída em 31 de agosto de 2026**

Esta versão deve transformar o dashboard estático atual em uma aplicação desktop modular, navegável e interativa.

A v0.4 ainda NÃO deve implementar:

* SQLite real;
* IA real;
* Ollama;
* automações;
* MQTT;
* IoT;
* controle real do Windows.

Esses recursos pertencem às próximas versões.

# Stack

Use:

* Tauri 2
* React
* TypeScript
* Vite
* Rust apenas para a camada Tauri
* CSS / CSS Modules
* SVG para elementos HUD e gráficos customizados

Não utilizar Tailwind.

Evitar bibliotecas pesadas sem necessidade.

# Referência visual

Existe no projeto um dashboard anterior do Azriel.

Use-o como principal referência visual.

Preserve a identidade:

* fundo escuro;
* cyan;
* azul petróleo;
* HUD técnico;
* linhas finas;
* círculos concêntricos;
* microtipografia;
* telemetria;
* gráficos técnicos;
* painéis assimétricos;
* alta densidade de informação;
* animações discretas.

Evite aparência de:

* landing page;
* dashboard SaaS;
* cards genéricos;
* interface típica gerada por IA;
* excesso de blur ou glow.

Queremos que pareça uma ferramenta pessoal de pesquisa e engenharia.

# Estrutura principal

Criar os seguintes módulos:

* Command Center
* Projetos
* Conhecimento
* Mapa Stark
* Formação
* Pesquisa
* Sistema
* Configurações

A aplicação deve ser pensada primeiro para desktop, especialmente 1920×1080.

# Azriel Core

Transformar o núcleo circular atual em um componente React chamado:

`AzrielCore`

Estados:

```ts
type AzrielState =
  | "idle"
  | "processing"
  | "alert"
  | "offline";
```

Nesta versão os estados podem ser simulados.

O núcleo deve possuir:

* anéis concêntricos;
* segmentos;
* ticks;
* movimento lento;
* pulsação;
* pequenas animações;
* hover;
* clique.

Ao clicar, abrir um painel com estado do sistema e módulos.

Usar preferencialmente CSS + SVG.

# Mapa Stark

Transformar o gráfico atual em componente React interativo.

Áreas iniciais:

* Cibersegurança
* Programação
* IA / Software
* Biologia / Genética
* Biomedicina / Biotecnologia
* Física Aplicada
* Energia
* Robótica
* Eletrônica
* Controle / Automação
* Materiais
* Engenharia Elétrica
* Engenharia Mecânica

Manter as métricas:

* Cobertura
* Profundidade

Ao passar o mouse:

mostrar detalhes.

Ao clicar:

abrir painel lateral com:

* cobertura;
* profundidade;
* lacuna;
* projetos relacionados;
* prioridade.

Não usar um gráfico genérico se isso quebrar a identidade HUD.

# Projetos

Criar estrutura de dados reutilizável para projetos.

Projetos iniciais:

* Azriel
* ArcCore
* Mendel Lab
* Gene Expression Explorer
* PCR Simulator
* GeneScope
* Atlas3D

Atlas3D deve aparecer como pausado.

Cada projeto deve possuir:

```ts
interface Project {
  id: string;
  name: string;
  category: string;
  description: string;
  status: "active" | "research" | "paused" | "planned";
  knowledgeAreas: string[];
  objective: string;
}
```

Criar uma tela ou painel de detalhes do projeto.

# Formação

Registrar:

## Pós-graduações em andamento

Conclusão prevista para novembro de 2026:

* Biotecnologia
* Internet das Coisas — IoT
* Big Data Analytics

## Próxima formação

Biomedicina — prevista para 2027.

## Formação futura

Engenharia Mecatrônica.

## Posteriormente

Mestrado interdisciplinar ainda não definido.

# Knowledge Core visual

Criar uma representação inicial das áreas:

* Computação
* Cibersegurança
* IA
* Biologia
* Genética
* Biologia Molecular
* Biotecnologia
* Bioinformática
* Matemática
* Física
* Eletrônica
* Mecânica
* Energia
* Controle
* Robótica
* IoT
* Big Data
* Materiais

Cada área pode utilizar dados mockados nesta versão.

Mostrar:

* cobertura;
* profundidade;
* prioridade;
* projetos relacionados.

# Diagnóstico de lacunas

Criar componente:

`GapDiagnostics`

Prioridades atuais:

Alta:

* Engenharia Elétrica
* Engenharia Mecânica
* Eletrônica
* Física Aplicada
* Matemática

Média:

* Robótica
* Controle
* Materiais

# Dados

Não espalhar valores diretamente nos componentes.

Centralizar mocks em algo como:

```text
src/data/
├── projects.ts
├── knowledge.ts
├── education.ts
├── starkMap.ts
└── system.ts
```

# Arquitetura

Organizar componentes por domínio.

Sugestão:

```text
src/
├── components/
│   ├── azriel/
│   ├── hud/
│   ├── stark/
│   ├── projects/
│   └── knowledge/
├── pages/
├── data/
├── types/
├── styles/
└── App.tsx
```

Pode adaptar se encontrar uma arquitetura melhor.

# Animações

Use movimento sutil:

* rotação lenta;
* pulsação;
* scan line;
* fade;
* barras animadas;
* hover técnico.

Respeitar:

`prefers-reduced-motion`

Evitar animações excessivas.

# Tauri

Configurar como aplicativo desktop Tauri 2.

Nome:

`Azriel`

Título da janela:

`AZRIEL // Personal Intelligence System`

Garantir que:

```bash
npm install
npm run tauri dev
```

funcione corretamente.

Não adicionar permissões nativas desnecessárias.

# Importante

Se houver código antigo do assistente local no repositório:

* preserve;
* não apague;
* não faça uma integração grande nesta versão;
* apenas deixe a arquitetura preparada para a futura AI Core.

# Critérios de aceite

A tarefa só está concluída quando:

1. o projeto executa como aplicação Tauri;
2. o visual preserva a identidade do dashboard atual;
3. existe navegação entre módulos;
4. `AzrielCore` está animado e interativo;
5. o Mapa Stark está interativo;
6. os projetos reais aparecem;
7. as três pós aparecem com conclusão em novembro de 2026;
8. Biomedicina aparece como próxima formação;
9. Mecatrônica aparece como formação futura;
10. o diagnóstico de lacunas existe;
11. mocks estão separados dos componentes;
12. TypeScript não apresenta erros;
13. o build funciona;
14. a interface funciona corretamente em 1920×1080;
15. a estrutura está preparada para SQLite na v0.5.

# Processo

Antes de implementar:

* faça uma leitura completa do repositório;
* descreva brevemente a arquitetura encontrada;
* apresente um plano curto;
* depois implemente.

Durante a implementação:

* rode lint;
* rode TypeScript;
* rode build;
* corrija os erros encontrados.

Ao terminar, informe:

* arquivos criados;
* arquivos modificados;
* dependências adicionadas;
* decisões arquiteturais;
* como executar;
* limitações atuais;
* próximos passos para a v0.5.

Não considere a tarefa concluída com build quebrado.

# Resultado esperado

A v0.4 deve deixar de parecer apenas um dashboard estático.

Ela deve parecer a primeira versão real da interface de um sistema pessoal de inteligência ainda em construção.

# Encerramento da versão

A v0.4 foi concluída com os 15 critérios de aceite atendidos.

## Entregas confirmadas

* aplicação desktop configurada e executada com Tauri 2;
* interface React e TypeScript com identidade HUD preservada;
* navegação entre oito módulos;
* menu lateral expansível e recolhível;
* tipografia revisada para maior legibilidade;
* `AzrielCore` animado, interativo e com estados simulados;
* Mapa Stark interativo com detalhes por área;
* projetos, conhecimento, formação, pesquisa e diagnóstico de lacunas;
* dados mockados centralizados e separados dos componentes;
* ícone oficial gerado para os formatos exigidos pelo Tauri;
* estrutura preparada para substituir mocks por SQLite na v0.5.

## Validações finais

* lint sem erros ou avisos;
* TypeScript sem erros;
* build de produção concluído;
* `cargo check` concluído;
* execução desktop Tauri confirmada;
* navegação e interações verificadas;
* interface verificada em 1920×1080;
* ausência de textos cortados e de erros no console.

## Limites preservados

A v0.4 não implementa SQLite, IA real, Ollama, automações, MQTT, IoT ou
controle do Windows. Esses limites são intencionais e permanecem reservados
para as próximas versões do roadmap.
