# AZRIEL — Engineering Core v0.3 — Model Core

A versão **Engineering Core v0.2 — Spatial Manipulation** foi concluída e validada.

MOVE, ROTATE e SCALE funcionam, embora a calibração ainda seja experimental e será refinada futuramente.

Agora implemente:

# Engineering Core v0.3 — Model Core

Antes de modificar qualquer arquivo:

1. leia o README;
2. leia `docs/engineering-core/v0.1.md`;
3. leia `docs/engineering-core/v0.2.md`;
4. examine toda a arquitetura atual do Engineering Core;
5. preserve câmera, hand tracking, Gesture Engine, modos MOVE / ROTATE / SCALE e calibração existente;
6. rode lint, testes e build;
7. não redesenhe o Engineering View sem necessidade.

---

# Objetivo

Substituir o objeto de teste simples por suporte a modelos 3D reais.

Ao final desta versão, o Engineering Core deve conseguir:

* carregar arquivos `.glb`;
* carregar arquivos `.gltf` quando tecnicamente viável;
* exibir o modelo no Engineering View;
* enquadrar o modelo automaticamente;
* mover o modelo;
* rotacionar o modelo;
* alterar sua escala;
* inspecionar informações básicas;
* descarregar/resetar modelo;
* lidar com erros de carregamento.

A interação gestual já existente deve continuar funcionando.

---

# Princípio central

A v0.3 deve provar:

# O ENGINEERING CORE CONSEGUE CARREGAR E MANIPULAR UM MODELO 3D REAL.

Não implementar ainda componentes internos complexos ou exploded view.

---

# Formatos

Prioridade:

```text
.glb
```

Suporte secundário:

```text
.gltf
```

GLB deve ser o formato preferencial porque encapsula melhor assets em um único arquivo.

Não implementar formatos CAD nativos como:

* STEP;
* IGES;
* STL avançado;
* SolidWorks;
* Fusion;
* Inventor.

Esses formatos poderão entrar futuramente por conversão.

---

# Stack

Continuar utilizando Three.js / React Three Fiber existente.

Usar loader apropriado:

```text
GLTFLoader
```

ou equivalente compatível com a arquitetura atual.

Não criar loader próprio.

---

# Model Service

Criar camada separada responsável por carregamento.

Exemplo conceitual:

```text
ModelService
```

Responsabilidades:

* abrir arquivo;
* validar extensão;
* carregar GLB/GLTF;
* retornar cena/modelo;
* tratar erro;
* extrair metadata;
* liberar recursos.

Não colocar lógica de loader diretamente no componente principal.

---

# Seleção de arquivo

Adicionar ação:

```text
CARREGAR MODELO
```

Utilizar diálogo nativo do Tauri quando apropriado.

Aceitar inicialmente:

```text
.glb
.gltf
```

Não permitir caminhos arbitrários vindos do AI Core.

A seleção é feita diretamente pelo operador.

---

# Privacidade

Modelos são carregados localmente.

Não enviar arquivos para:

* Ollama;
* APIs;
* cloud;
* serviços externos.

Mostrar:

```text
MODEL PROCESSING
LOCAL
```

---

# Estado do Model Core

Criar estados:

```text
empty
loading
ready
error
```

Exemplo:

```text
MODEL CORE
SEM MODELO
```

durante vazio.

```text
MODEL CORE
CARREGANDO
```

durante load.

```text
MODEL CORE
ONLINE
```

quando pronto.

---

# Auto framing

Após carregar o modelo:

calcular bounding box.

Usar:

```text
Box3
```

ou equivalente.

Determinar:

* centro;
* tamanho;
* dimensões.

Reposicionar câmera ou modelo para que o objeto fique corretamente enquadrado.

Não assumir escala fixa.

---

# Normalização

Modelos podem vir com escalas muito diferentes.

Criar normalização inicial para visualização.

Exemplo conceitual:

```text
bounding box
↓
largest dimension
↓
normalized display scale
```

Preservar escala original como metadata.

Não destruir transformações internas do asset sem necessidade.

---

# Scene wrapper

O modelo carregado deve ser colocado dentro de um container/grupo controlável.

Exemplo:

```text
ModelRoot
   ↓
GLTF Scene
```

MOVE / ROTATE / SCALE devem atuar sobre:

```text
ModelRoot
```

e não modificar individualmente cada mesh.

Isso será importante para exploded view futuramente.

---

# Manter TEST-01

O objeto TEST-01 pode permanecer como fallback/debug.

Mas quando um modelo real estiver carregado:

ele deve substituir TEST-01 como objeto principal.

---

# Metadata

Extrair informações básicas.

Mostrar no inspector:

```text
MODEL

NAME
motor.glb

FORMAT
GLB

OBJECTS
24

MESHES
18

MATERIALS
6

DIMENSIONS
X ...
Y ...
Z ...

STATUS
READY
```

Somente mostrar métricas que puderem ser obtidas de forma confiável.

---

# Scene Traversal

Percorrer a cena carregada para identificar:

* Objects;
* Groups;
* Meshes;
* Materials.

Não implementar ainda seleção individual de componentes.

Mas preparar estrutura de dados para isso.

---

# Model Node

Criar representação conceitual:

```ts
interface ModelNode {
  id: string;
  name: string;
  type: string;
  parentId?: string;
  children: string[];
}
```

Adaptar se necessário.

Objetivo:

preparar a próxima versão.

---

# Model Hierarchy

Adicionar painel opcional:

```text
MODEL STRUCTURE

MotorAssembly
├── Housing
├── Rotor
├── Shaft
└── Connector
```

Nesta versão:

somente visualização.

Não precisa selecionar componentes ainda.

Se o modelo não possuir nomes úteis:

usar nomes técnicos gerados de forma estável.

---

# Wireframe mode

Adicionar opção:

```text
WIREFRAME
ON / OFF
```

Quando ativo:

mostrar o modelo em modo wireframe quando tecnicamente possível.

Preservar materiais originais para restauração.

---

# Grid

Preservar grid técnico atual.

Adicionar opção:

```text
GRID
ON / OFF
```

---

# Axes

Adicionar opcionalmente:

```text
AXES
ON / OFF
```

Mostrar eixo XYZ pequeno.

Útil para engenharia.

---

# Transform Inspector

Preservar e atualizar:

```text
POSITION
X
Y
Z

ROTATION
X
Y
Z

SCALE
```

Esses valores agora devem refletir `ModelRoot`.

---

# Gestos

Preservar:

```text
MOVE
ROTATE
SCALE
```

Não alterar a linguagem gestual atual nesta versão.

Mesmo que existam problemas de calibração.

Registrar calibração como limitação conhecida.

---

# Mouse fallback

Adicionar ou preservar controles básicos com mouse.

O usuário deve conseguir manipular o modelo mesmo sem webcam.

Sugestão:

* drag → orbit camera;
* wheel → zoom;
* selecionar modo;
* reset.

Não precisa replicar exatamente os gestos.

O objetivo é permitir depuração.

---

# Camera Orbit

Se já existir orbit controls, preservar.

Caso não exista, adicionar `OrbitControls` ou equivalente.

Mas evitar conflito com hand interaction.

Quando o objeto estiver sendo manipulado pela mão:

camera orbit não deve interferir.

---

# Reset Model

Adicionar:

```text
RESET MODEL
```

Restaurar:

* posição inicial;
* rotação inicial;
* escala inicial;
* camera framing.

---

# Unload Model

Adicionar:

```text
DESCARREGAR MODELO
```

Ao remover:

* liberar geometrias;
* liberar materiais quando seguro;
* liberar texturas;
* limpar referências;
* voltar ao estado vazio.

Evitar memory leak.

---

# Resource cleanup

Three.js assets podem consumir bastante memória.

Implementar cleanup adequado.

Ao trocar de modelo:

descarregar o anterior.

Não manter modelos antigos escondidos na cena.

---

# Loading UI

Modelos maiores podem demorar.

Mostrar:

```text
CARREGANDO MODELO...
```

Se progresso estiver disponível de forma confiável:

mostrar porcentagem.

Caso contrário, não inventar progresso.

---

# Erros

Tratar:

* arquivo inválido;
* GLB corrompido;
* GLTF incompleto;
* textura ausente;
* loader failure;
* modelo vazio;
* memória insuficiente quando detectável.

Mostrar erro técnico curto e compreensível.

---

# GLTF externo

Arquivos `.gltf` podem depender de:

* `.bin`;
* texturas externas.

Se a arquitetura local tornar isso complexo:

documentar limitação.

Priorizar funcionamento perfeito de GLB.

Não quebrar v0.3 tentando suportar todo caso possível de GLTF.

---

# HUD atual

Atualizar os estados existentes.

Quando nenhum modelo:

```text
MODELO
NÃO CARREGADO
```

Quando carregado:

```text
MODELO
ONLINE
```

Renderer:

```text
RENDERIZADOR 3D
ONLINE
```

Hand tracking mantém estado atual.

---

# CAD

Manter:

```text
CAD
NÃO CONECTADO
```

GLB/GLTF não significa CAD nativo.

Não renomear isso de forma incorreta.

---

# Digital Twin

Manter:

```text
DIGITAL TWIN
NÃO CONECTADO
```

---

# IoT

Manter:

```text
IOT
NÃO CONECTADO
```

---

# Performance

Modelos reais podem ser pesados.

Evitar:

* traversal a cada frame;
* recriação de materiais;
* React state excessivo;
* recalcular bounding box constantemente.

Calcular metadata apenas quando necessário.

---

# Limites

Se for simples, adicionar warning para modelos muito grandes.

Exemplo:

```text
HIGH COMPLEXITY MODEL
```

baseado em quantidade de triangles/meshes.

Não bloquear automaticamente sem necessidade.

---

# Model Stats

Se puder ser obtido eficientemente:

mostrar:

```text
VERTICES
TRIANGLES
```

Não tornar obrigatório se exigir processamento excessivo.

---

# Persistência

Não é obrigatório reabrir automaticamente o último modelo após reiniciar Azriel.

Pode salvar apenas:

* último caminho;
* preferências visuais;
* wireframe;
* grid.

Mas não carregar automaticamente sem decisão explícita se isso aumentar tempo de inicialização.

---

# Segurança

Engineering Core não deve:

* alterar o modelo original no disco;
* sobrescrever arquivo;
* converter arquivo automaticamente;
* executar scripts embutidos;
* enviar asset para LLM.

O modelo é somente leitura nesta versão.

---

# AI Core

Continuar sem integração funcional.

Não permitir comandos como:

```text
Azriel, carregue C:\...
```

ainda.

Carregamento deve vir da UI.

Preparar interfaces internas futuras:

```text
get_loaded_model
get_model_structure
reset_model
```

mas não expor como tools de escrita/ação ainda.

---

# Testes

Adicionar testes para lógica isolável:

1. extensão GLB válida;
2. extensão GLTF válida;
3. arquivo inválido;
4. bounding box;
5. centro;
6. dimensões;
7. escala normalizada;
8. metadata;
9. scene traversal;
10. model hierarchy;
11. reset transform;
12. unload;
13. cleanup;
14. state transitions.

Não depender de modelo externo em todos os testes.

Adicionar fixture 3D mínima se necessário.

---

# Documentação

Criar:

```text
docs/engineering-core/v0.3.md
```

Documentar:

* objetivo;
* Model Core;
* formatos;
* GLTFLoader;
* auto framing;
* normalização;
* ModelRoot;
* hierarchy;
* metadata;
* cleanup;
* limitações do GLTF externo;
* performance.

Registrar:

```text
v0.1 — Hand Interaction — concluída
v0.2 — Spatial Manipulation — concluída
v0.3 — Model Core
```

Também registrar:

```text
Known limitation:
calibração do tracking ainda requer refinamento.
```

---

# Critérios de aceite

Engineering Core v0.3 só está concluído quando:

1. GLB puder ser selecionado;
2. GLB puder ser carregado;
3. modelo aparecer corretamente;
4. auto framing funcionar;
5. dimensões forem calculadas;
6. metadata básica existir;
7. hierarquia puder ser inspecionada;
8. MOVE funcionar no modelo;
9. ROTATE funcionar no modelo;
10. SCALE funcionar no modelo;
11. hand tracking continuar funcionando;
12. wireframe funcionar;
13. grid puder ser ativado/desativado;
14. axes puderem ser ativados/desativados quando implementado;
15. reset funcionar;
16. unload funcionar;
17. carregar segundo modelo substituir o anterior;
18. recursos do modelo anterior forem liberados;
19. arquivo inválido for tratado;
20. renderer não quebrar sem modelo;
21. CAD permanecer marcado como não conectado;
22. Digital Twin permanecer não conectado;
23. IoT permanecer não conectado;
24. TypeScript não apresentar erros;
25. testes passarem;
26. build funcionar;
27. Tauri iniciar normalmente;
28. demais módulos do Azriel não sofrerem regressão.

---

# Teste final obrigatório

Abrir:

```text
Engineering Core
```

Selecionar:

```text
CARREGAR MODELO
```

Escolher um arquivo:

```text
.glb
```

Confirmar:

```text
MODEL CORE ONLINE
```

Verificar:

* modelo visível;
* framing correto;
* metadata;
* hierarchy.

Testar:

```text
MOVE
```

Manipular com pinch.

Testar:

```text
ROTATE
```

Testar:

```text
SCALE
```

Confirmar que os três continuam funcionando.

Ativar:

```text
WIREFRAME
```

Desativar.

Executar:

```text
RESET MODEL
```

Confirmar transform inicial.

Carregar outro modelo.

Confirmar que o anterior foi removido.

Executar:

```text
DESCARREGAR MODELO
```

Confirmar retorno ao estado vazio.

---

# Resultado esperado

Ao concluir esta versão, o Engineering Core deixa de ser apenas uma demonstração de interação gestual.

Ele passa a ser um visualizador e manipulador de modelos 3D reais.

A evolução deve ficar clara:

```text
v0.1
mão controla cubo

v0.2
mão move, gira e escala

v0.3
mão manipula modelo 3D real
```

A próxima versão poderá finalmente começar a tratar o modelo não como um único objeto, mas como um conjunto de componentes de engenharia.

---

# Estado da implementação — 01/09/2026

Status: **Engineering Core v0.3 concluída e aprovada em 01/09/2026**.

Foram implementados:

* seleção local de GLB/GLTF pelo diálogo nativo;
* leitura validada pelo backend, limitada a 100 MB;
* ModelService com GLTFLoader, estados e erros explícitos;
* GLB como formato principal e GLTF incorporado como suporte secundário;
* bounding box, centro, dimensões e escala normalizada;
* ModelRoot independente da cena interna do asset;
* auto framing e OrbitControls;
* metadata, estatísticas e hierarquia somente para leitura;
* MOVE, ROTATE e SCALE aplicados ao modelo carregado;
* wireframe, grid e axes configuráveis;
* reset, substituição e descarregamento com cleanup;
* TEST-01 preservado como fallback;
* testes automatizados do Model Core;
* documentação em Docs/engineering-core/v0.3.md.

## Limitações registradas

GLTF que depende de .bin ou texturas externas não é resolvido na v0.3. Nesses casos, o Engineering Core solicita o uso de GLB ou de GLTF com recursos incorporados.

A calibração automática do tracking continua reservada para **v0.2.1**, conforme encerramento aprovado da v0.2.

## Validação operacional concluída

O aplicativo Tauri iniciou normalmente e a interface do Model Core foi aprovada pelo operador. A versão fica encerrada com suporte prioritário a GLB, manipulação do ModelRoot, metadata, hierarchy, controles visuais e cleanup.

Permanecem como limitações conhecidas, sem bloquear o aceite:

* GLTF com arquivos auxiliares externos;
* codecs Draco, KTX2 e Meshopt;
* calibração automática do tracking, reservada para v0.2.1;
* seleção de componentes, exploded view e CAD nativo, destinados a versões futuras.
