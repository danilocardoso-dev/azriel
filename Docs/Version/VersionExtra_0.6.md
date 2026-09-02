# AZRIEL — Engineering Core v0.6 — AI Integration

A versão **Engineering Core v0.5 — Exploded View** foi concluída e validada.

O Engineering Core já possui:

* webcam;
* hand tracking;
* gestos;
* MOVE / ROTATE / SCALE;
* carregamento GLB/GLTF;
* Model Core;
* Component Core;
* seleção individual de componentes;
* inspector;
* hide/show;
* isolamento;
* Exploded View;
* reconstrução;
* controle por duas mãos.

Agora implemente:

# Engineering Core v0.6 — AI Integration

---

# Objetivo

Conectar o **AI Core existente do Azriel** ao Engineering Core.

Ao final desta versão, o operador deve conseguir conversar com o Azriel sobre o modelo 3D carregado e executar operações controladas através de linguagem natural.

Exemplos:

```text
Azriel, qual modelo está carregado?
```

```text
Quantos componentes ele possui?
```

```text
Encontre o rotor.
```

```text
Destaque o rotor.
```

```text
Isole o conjunto do eixo.
```

```text
Exploda a montagem.
```

```text
Abra em 60%.
```

```text
Reconstrua a montagem.
```

As ações devem utilizar APIs internas controladas.

O LLM não pode manipular Three.js diretamente.

---

# Antes de modificar código

1. leia o README;
2. leia `docs/engineering-core/v0.3.md`;
3. leia `docs/engineering-core/v0.4.md`;
4. leia `docs/engineering-core/v0.5.md`;
5. leia a documentação atual do AI Core;
6. entenda Tool Registry e Tool Router existentes;
7. preserve a arquitetura de segurança atual;
8. preserve Engineering Core funcional;
9. preserve Automation Core funcional;
10. rode lint, testes e build antes de iniciar.

Não refatore módulos estáveis sem necessidade.

---

# Princípio central

A arquitetura deve ser:

```text
OPERADOR
   ↓
AI CORE
   ↓
ENGINEERING TOOLS
   ↓
ENGINEERING SERVICE
   ↓
MODEL / COMPONENT / EXPLOSION CORE
```

Nunca:

```text
LLM
↓
Three.js direto
```

---

# Separação entre leitura e ação

Criar duas categorias de Engineering Tools.

## READ

Podem ser utilizadas livremente pelo AI Core:

```text
get_loaded_model
get_model_summary
list_components
find_component
get_component_details
get_selected_component
get_explosion_state
```

## ACTION

Podem alterar apenas o estado visual da sessão 3D:

```text
select_component
focus_component
isolate_component
show_all_components
hide_component
show_component
set_explosion_factor
explode_all
explode_component
reassemble
reset_model_view
```

Essas ações não modificam arquivos.

---

# Segurança

Engineering ACTION tools podem:

* alterar seleção;
* alterar visibilidade;
* alterar câmera;
* alterar transform visual;
* alterar Exploded View.

Não podem:

* editar GLB;
* salvar GLB;
* sobrescrever arquivo;
* modificar geometria;
* executar código;
* acessar filesystem arbitrário;
* executar shell;
* chamar Automation Core arbitrariamente;
* alterar IoT;
* alterar Windows.

---

# Engineering Tool Registry

Adicionar tools ao Tool Registry já existente.

Não criar sistema paralelo de ferramentas.

Exemplo conceitual:

```ts
interface EngineeringTool {
  name: string;
  description: string;
  permission: "read" | "visual_action";
  execute(input: unknown): Promise<unknown>;
}
```

Adaptar à arquitetura atual.

Evitar `any`.

---

# Session Context

Engineering Core deve expor estado controlado da sessão.

Criar estrutura semelhante:

```ts
interface EngineeringSessionState {
  modelLoaded: boolean;
  modelName?: string;

  componentCount: number;
  selectedComponentId?: string;

  explosionFactor: number;
  explosionMode: "all" | "selected";

  modelMode: "model" | "component";
}
```

O AI Core não precisa receber a scene graph Three.js inteira.

---

# get_loaded_model

Retornar informações básicas:

```text
name
format
components
meshes
materials
dimensions
status
```

Não retornar objetos Three.js serializados.

---

# get_model_summary

Criar um resumo estruturado.

Exemplo:

```json
{
  "name": "motor.glb",
  "components": 24,
  "meshes": 18,
  "materials": 6,
  "selectedComponent": null,
  "explosionFactor": 0
}
```

---

# list_components

Retornar lista resumida.

Exemplo:

```json
[
  {
    "id": "component-01",
    "name": "Rotor",
    "type": "Mesh",
    "parent": "MotorAssembly"
  }
]
```

Evitar enviar metadata pesada sem necessidade.

---

# find_component

Entrada:

```text
query
```

Exemplo:

```text
rotor
```

Buscar por:

* nome exato;
* case insensitive;
* partial match.

Retornar candidatos.

Não deixar o LLM inventar IDs.

---

# Ambiguidade

Se houver mais de um componente compatível:

não escolher silenciosamente.

Exemplo:

```text
Rotor_Left
Rotor_Right
```

Azriel deve perguntar qual deles.

Ou listar as opções.

---

# get_component_details

Entrada:

```text
componentId
```

Retornar:

* nome;
* tipo;
* parent;
* children;
* visible;
* selected;
* transform;
* materials;
* bounding box quando disponível.

---

# get_selected_component

Retornar o componente atualmente selecionado pelo usuário através de:

* mouse;
* árvore;
* gesto;
* AI.

Isso deve permitir perguntas como:

```text
Azriel, o que é essa peça?
```

Nesse caso, o AI Core utiliza o componente selecionado como contexto.

---

# Referências contextuais

Suportar expressões:

```text
essa peça
esse componente
a peça selecionada
isso
```

somente quando existir `selectedComponentId`.

Se não existir:

responder:

```text
Nenhum componente está selecionado.
```

Não adivinhar.

---

# select_component

Entrada:

```text
componentId
```

Executar através do Component Core existente.

Deve produzir o mesmo resultado da seleção manual.

---

# focus_component

Entrada:

```text
componentId
```

Usar funcionalidade já existente.

Não criar implementação duplicada.

---

# isolate_component

Entrada:

```text
componentId
```

Usar Component Service / Core.

---

# show_all_components

Restaurar visibilidade apropriada.

Preservar regras existentes de visibilidade quando aplicável.

---

# hide_component

Entrada:

```text
componentId
```

Apenas visual.

---

# show_component

Entrada:

```text
componentId
```

---

# get_explosion_state

Retornar:

```text
enabled
factor
mode
selectedRoot
```

---

# set_explosion_factor

Entrada:

```ts
{
  factor: number;
}
```

Validar:

```text
0 <= factor <= 1
```

Não aceitar fora do intervalo.

---

# Linguagem natural para fator

Tool Router deve compreender frases como:

```text
abra em 50%
```

→

```text
0.5
```

```text
abra um pouco mais
```

Pode utilizar incremento controlado, por exemplo:

```text
+0.15
```

se houver estado atual conhecido.

```text
feche um pouco
```

→ redução equivalente.

Não deixar o LLM inventar valores extremos.

---

# explode_all

Comportamento:

usar Explosion Core existente.

Pode definir:

```text
factor = 1
```

ou iniciar animação já existente.

---

# explode_component

Entrada:

```text
componentId
```

Selecionar subárvore quando suportado.

Se componente não tiver filhos úteis:

retornar erro compreensível.

---

# reassemble

Comportamento:

```text
factor → 0
```

Usar implementação existente.

---

# reset_model_view

Restaurar:

* camera framing;
* model transform global;
* seleção visual quando apropriado.

Não descarregar modelo.

---

# Tool Router

Expandir o roteamento existente.

O modelo local inicial continua sendo pequeno.

Portanto, não depender exclusivamente de function calling sofisticado.

Usar estratégia híbrida:

```text
regras
+
aliases
+
matching
+
LLM
```

---

# Intentos principais

Criar suporte confiável para:

```text
model_status
component_search
component_select
component_inspect
component_focus
component_isolate
component_visibility
assembly_explode
assembly_reassemble
explosion_adjust
```

---

# Exemplos obrigatórios

## Modelo

```text
Qual modelo está carregado?
```

→ `get_loaded_model`

---

## Componentes

```text
Quantos componentes existem?
```

→ `get_model_summary`

---

## Busca

```text
Encontre o rotor.
```

→ `find_component`

---

## Seleção

```text
Destaque o rotor.
```

Fluxo:

```text
find_component
↓
resolve único candidato
↓
select_component
```

---

## Isolamento

```text
Isole o rotor.
```

→ `isolate_component`

---

## Explosão

```text
Exploda a montagem.
```

→ `explode_all`

---

## Parcial

```text
Abra em 60%.
```

→ `set_explosion_factor(0.6)`

---

## Incremental

```text
Abra mais.
```

→ ler `get_explosion_state`

→ aumentar fator de forma controlada.

---

## Reconstrução

```text
Reconstrua.
```

→ `reassemble`

---

# Cross-context

Adicionar consultas que combinem AI + Engineering.

Exemplo:

```text
Qual é a peça selecionada?
```

```text
Quantos filhos ela possui?
```

```text
Esse componente está visível?
```

---

# Estado visual do Azriel

Quando AI Core acionar Engineering Core:

AzrielCore pode mostrar:

```text
ENGINEERING COMMAND
```

ou:

```text
MANIPULANDO MODELO
```

Manter animação discreta.

---

# Feedback no Engineering View

Ao executar comando por IA, mostrar log curto:

```text
AZRIEL COMMAND

SELECT COMPONENT
Rotor
SUCCESS
```

Não transformar isso em terminal extenso.

---

# Confirmation

Ações desta versão são apenas visuais e reversíveis.

Podem ser classificadas como:

```text
VISUAL_ACTION
```

Não exigir confirmação para:

* seleção;
* foco;
* isolamento;
* exploded view;
* hide/show.

Mas manter arquitetura preparada para ações futuras mais importantes.

---

# Sem modelo

Se não houver modelo carregado:

```text
Exploda a montagem.
```

deve responder:

```text
Nenhum modelo está carregado no Engineering Core.
```

Não quebrar.

---

# Sem componente

Se busca não encontrar:

```text
Encontre o rotor.
```

responder:

```text
Não encontrei nenhum componente chamado "rotor" no modelo atual.
```

---

# Modelo com nomes ruins

Modelos podem possuir:

```text
Mesh_001
Mesh_002
```

Nesse caso:

não inventar nomes semânticos.

Azriel pode responder:

```text
O modelo não possui nomes descritivos suficientes para identificar um rotor.
```

Isso é crítico.

---

# Semântica

A v0.6 NÃO deve fingir compreender engenharia mecânica apenas por olhar nomes de meshes.

Se o node se chama:

```text
Rotor
```

pode usar esse nome.

Se se chama:

```text
Mesh_047
```

não chamar de rotor por inferência visual.

---

# Preparação para Assembly Intelligence

Criar campo opcional em memória:

```ts
semanticLabel?: string
```

mas não preencher automaticamente nesta versão.

Será usado futuramente.

---

# Histórico de Engineering Commands

Adicionar log de sessão.

Exemplo:

```text
14:32 select_component Rotor AI
14:33 set_explosion_factor 0.6 AI
14:34 reassemble AI
```

Pode ser somente sessão em memória inicialmente.

Persistência não é obrigatória.

---

# AI Conversation

Manter conversas existentes.

Engineering commands devem ocorrer dentro da mesma interface atual do Azriel.

Não criar chatbot separado.

---

# Prompt de sistema

Atualizar prompt do AI Core com instruções curtas:

* Engineering Core existe;
* usar tools quando necessário;
* não inventar componentes;
* não inventar propriedades;
* confirmar ambiguidade;
* não afirmar sem dados;
* diferenciar nome técnico do modelo de interpretação semântica.

Não colocar documentação inteira no system prompt.

---

# Performance

Não enviar ao modelo toda a hierarchy em todas as mensagens.

Usar Context Builder.

Exemplo:

```text
"Quantos componentes?"
```

→ summary apenas.

```text
"Liste todos."
```

→ list_components.

```text
"Detalhes do rotor."
```

→ apenas rotor.

---

# Modelo pequeno

O modelo padrão atual pode continuar sendo:

```text
qwen2.5:0.5b
```

Portanto:

* manter respostas estruturadas;
* evitar contexto gigante;
* não depender do modelo para cálculos;
* resolver IDs e componentes no software.

---

# Tool Result Format

Retornos devem ser claros e compactos.

Evitar enviar objetos complexos desnecessários.

---

# Segurança

AI Engineering Tools NÃO podem:

* abrir arquivos arbitrários;
* carregar modelo por caminho fornecido pelo LLM;
* modificar arquivo;
* exportar;
* executar shell;
* acessar IoT;
* executar Automation Core;
* alterar sistema.

Carregar modelo continua sendo ação manual da UI nesta versão.

---

# Testes

Adicionar testes para:

1. get_loaded_model;
2. get_model_summary;
3. list_components;
4. find exact;
5. find partial;
6. ambiguous search;
7. no result;
8. selected component;
9. select component;
10. isolate component;
11. hide/show;
12. explosion state;
13. factor clamp;
14. explode all;
15. explode selected;
16. leaf error;
17. reassemble;
18. no model;
19. poor component names;
20. Tool Router intents;
21. contextual "essa peça";
22. read/action permissions;
23. Fake Engineering Core.

---

# FakeEngineeringService

Criar fake para testes.

Exemplo:

```text
MotorAssembly
├── Housing
├── Rotor
├── Shaft
└── Bearing
```

Assim testes do AI Core não dependem de Three.js real.

---

# Testes obrigatórios de linguagem natural

Testar:

```text
Qual modelo está carregado?
```

```text
Quantos componentes ele possui?
```

```text
Encontre o rotor.
```

```text
Selecione o rotor.
```

```text
Isole o rotor.
```

```text
Exploda a montagem.
```

```text
Abra em 50%.
```

```text
Abra mais.
```

```text
Feche um pouco.
```

```text
Reconstrua a montagem.
```

```text
Qual peça está selecionada?
```

---

# Documentação

Criar:

```text
docs/engineering-core/v0.6.md
```

Documentar:

* objetivo;
* AI integration;
* Engineering Tool Registry;
* read tools;
* visual actions;
* Tool Router;
* Context Builder;
* component resolution;
* ambiguity;
* segurança;
* limitações semânticas;
* modelo local.

Registrar:

```text
v0.1 — Hand Interaction — concluída
v0.2 — Spatial Manipulation — concluída
v0.3 — Model Core — concluída
v0.4 — Component Core — concluída
v0.5 — Exploded View — concluída
v0.6 — AI Integration
```

Manter limitação:

```text
calibração gestual ainda requer refinamento.
```

---

# Critérios de aceite

Engineering Core v0.6 só está concluído quando:

1. AI Core reconhecer Engineering Core;
2. Engineering Tool Registry existir;
3. tools READ funcionarem;
4. visual actions funcionarem;
5. get_loaded_model funcionar;
6. get_model_summary funcionar;
7. list_components funcionar;
8. find_component funcionar;
9. ambiguidade for tratada;
10. selected component puder ser consultado;
11. AI puder selecionar componente;
12. AI puder focar componente;
13. AI puder isolar componente;
14. AI puder hide/show;
15. AI puder consultar explosion state;
16. AI puder explode all;
17. AI puder explode selected;
18. AI puder alterar explosion factor;
19. AI puder reassemble;
20. referência "essa peça" funcionar quando houver seleção;
21. referência sem seleção retornar erro adequado;
22. nomes inexistentes não forem inventados;
23. Mesh_### não receber semântica falsa;
24. Context Builder limitar dados;
25. qwen2.5:0.5b continuar utilizável;
26. modelo ausente for tratado;
27. AI não carregar arquivo arbitrariamente;
28. AI não modificar GLB;
29. AI não acessar shell;
30. TypeScript não apresentar erros;
31. testes passarem;
32. build funcionar;
33. Tauri iniciar normalmente;
34. módulos existentes não sofrerem regressão.

---

# Teste final obrigatório

Carregar manualmente um modelo GLB com múltiplos componentes.

No AI Core perguntar:

```text
Azriel, qual modelo está carregado?
```

Confirmar resposta real.

Depois:

```text
Quantos componentes existem?
```

Depois:

```text
Encontre [nome real de um componente].
```

Confirmar componente correto.

Depois:

```text
Destaque esse componente.
```

Confirmar seleção visual.

Depois:

```text
Isole essa peça.
```

Confirmar isolamento.

Depois:

```text
Mostre todos os componentes.
```

Confirmar restauração.

Executar:

```text
Exploda a montagem.
```

Confirmar Exploded View.

Depois:

```text
Abra em 50%.
```

Confirmar fator 0.5.

Depois:

```text
Abra mais.
```

Confirmar aumento controlado.

Depois:

```text
Reconstrua a montagem.
```

Confirmar factor 0.

Selecionar manualmente outra peça usando mouse ou gesto.

Perguntar:

```text
Azriel, qual peça está selecionada?
```

Confirmar sincronização entre Engineering Core e AI Core.

Finalmente testar nome inexistente.

Azriel deve admitir que não encontrou.

---

# Resultado esperado

Ao concluir a v0.6, o Engineering Core passa a possuir três formas de interação complementares:

```text
MOUSE
↓
PRECISÃO

MÃOS
↓
INTERAÇÃO ESPACIAL

AI CORE
↓
INTENÇÃO SEMÂNTICA
```

O operador poderá manipular fisicamente o modelo com as mãos e utilizar linguagem natural para operações de alto nível.

Exemplo:

```text
"Azriel, destaque o rotor."

"Agora isole essa peça."

"Exploda a montagem em 70%."

"Reconstrua."
```

A filosofia desta versão é:

# INTENÇÃO EM LINGUAGEM NATURAL, EXECUÇÃO POR FERRAMENTAS CONTROLADAS.

O LLM interpreta.

O Engineering Core executa.

O modelo 3D continua protegido.

---

# Estado da implementação — 02/09/2026

Status: **concluída e validada em 02/09/2026**.

Foram entregues a sessão compartilhada do Engineering Core, ferramentas `READ` e `VISUAL_ACTION`, roteamento controlado pelo AI Core, sincronização de seleção e estado visual, ações de foco, isolamento, visibilidade, explosão, reconstrução e reset, além do registro `AZRIEL COMMAND` para ações originadas pela IA.

O teste operacional foi aprovado no aplicativo Tauri com modelo GLB real. O carregamento permaneceu manual, o modelo continuou disponível ao alternar entre Engineering View e AI Core e os comandos em linguagem natural utilizaram exclusivamente as ferramentas registradas.

A calibração gestual continua como limitação conhecida e permanece planejada para a v0.2.1. A sessão 3D permanece somente em memória e é encerrada ao fechar o Azriel.

Com a aprovação do operador, a Engineering Core v0.6 está encerrada.
