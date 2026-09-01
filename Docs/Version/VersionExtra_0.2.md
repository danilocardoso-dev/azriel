# AZRIEL — Engineering Core v0.2

A versão **Engineering Core v0.1** foi concluída e validada.

A webcam detecta a mão, o gesto de pinça funciona e um objeto 3D pode ser selecionado, movimentado e solto.

Agora implemente:

# Engineering Core v0.2 — Spatial Manipulation

Antes de modificar qualquer arquivo:

1. leia o README;
2. leia `docs/engineering-core/v0.1.md`;
3. examine a arquitetura atual do Engineering Core;
4. preserve a implementação funcional de câmera, hand tracking, pinch, smoothing e cena 3D;
5. execute lint, testes e build;
6. não redesenhe a tela sem necessidade;
7. não refatore código estável apenas por preferência.

---

# Objetivo

Expandir a linguagem gestual do Engineering Core.

Ao final desta versão, o operador deve conseguir:

* detectar duas mãos;
* mover um objeto;
* rotacionar um objeto;
* aumentar e diminuir sua escala;
* apontar para o objeto;
* abrir a mão;
* calibrar a interação;
* visualizar claramente o estado de cada gesto.

A interação deve continuar exclusivamente local.

---

# Princípio central

A v0.2 deve provar:

# DUAS MÃOS CONSEGUEM MANIPULAR UM OBJETO 3D DE FORMA NATURAL.

Não aumentar o escopo para modelos complexos.

Continuar utilizando o objeto de teste atual.

---

# Fora do escopo

Não implementar nesta versão:

* GLB;
* GLTF;
* CAD;
* modelos reais;
* exploded view;
* seleção de componentes internos;
* física;
* Digital Twin;
* IoT;
* AI Core;
* comandos de voz;
* controle do Windows.

---

# Hand Tracking

Expandir o tracking de:

```text
1 mão
```

para:

```text
até 2 mãos
```

Representar cada mão separadamente.

Exemplo conceitual:

```ts
interface TrackedHand {
  id: string;
  handedness: "left" | "right";
  landmarks: HandLandmark[];
  confidence?: number;
}
```

Não depender apenas da ordem em que o MediaPipe retorna as mãos.

Usar handedness quando disponível.

---

# Estados das mãos

HUD deve conseguir mostrar:

```text
LEFT HAND
DETECTED

RIGHT HAND
DETECTED
```

ou:

```text
LEFT HAND
NOT DETECTED
```

---

# Gesture Engine

Expandir os gestos existentes.

Atualmente:

```text
NONE
PINCH
```

Adicionar:

```text
POINT
OPEN_HAND
```

Preparar arquitetura para futuros:

```text
GRAB
ROTATE
```

sem necessariamente criar classificação complexa por machine learning.

---

# POINT

Reconhecer um gesto simples de apontar.

Sugestão:

* indicador estendido;
* demais dedos relativamente fechados.

Não exigir perfeição anatômica.

Priorizar estabilidade.

POINT deve ser utilizado principalmente para seleção/hover.

---

# OPEN_HAND

Detectar mão aproximadamente aberta:

* vários dedos estendidos;
* distância razoável entre landmarks.

Usar como gesto neutro ou de liberação.

Não precisa ter precisão absoluta.

---

# PINCH

Preservar comportamento validado da v0.1.

Não quebrar:

```text
pinch → grab
release → soltar
```

---

# Máquina de estados gestual

Evitar que vários gestos sejam ativados simultaneamente de forma contraditória.

Criar prioridade ou state machine.

Exemplo:

```text
PINCH
tem prioridade sobre
POINT
```

quando polegar + indicador estiverem fechados.

---

# Manipulação 1 — Translation

Preservar movimentação atual.

Fluxo:

```text
PINCH sobre objeto
↓
GRABBED
↓
movimento da mão
↓
TRANSLATE
```

Adicionar limites razoáveis para impedir que o objeto desapareça muito longe da área de trabalho.

---

# Manipulação 2 — Scale

Adicionar escala utilizando duas mãos.

Fluxo sugerido:

```text
mão esquerda detectada
+
mão direita detectada
+
ambas em estado apropriado
↓
calcular distância entre mãos
↓
comparar com distância inicial
↓
alterar escala do objeto
```

Exemplo:

```text
mãos se afastam
→ escala aumenta

mãos se aproximam
→ escala diminui
```

---

# Scale Session

Quando a operação começar:

registrar:

```text
initialHandDistance
initialObjectScale
```

Durante a operação:

```text
scaleFactor =
currentHandDistance / initialHandDistance
```

Aplicar limites.

Exemplo conceitual:

```text
MIN_SCALE
MAX_SCALE
```

Não deixar objeto ficar infinitamente pequeno ou grande.

---

# Escala uniforme

Nesta versão usar:

```text
x = y = z
```

Não permitir deformação independente dos eixos.

---

# Smoothing da escala

Aplicar suavização.

Evitar que pequenas oscilações das mãos façam o objeto pulsar.

---

# Manipulação 3 — Rotation

Adicionar rotação de uma mão.

Não tentar estimar rotação 3D completa do punho de forma excessivamente complexa.

Criar uma primeira interação previsível.

Sugestão:

durante modo de rotação, usar deslocamento horizontal/vertical da mão.

Exemplo:

```text
mover mão horizontalmente
→ rotate Y

mover mão verticalmente
→ rotate X
```

Pode utilizar um gesto específico para ativação.

Sugestão inicial:

```text
PINCH + modifier/mode
```

ou utilizar uma zona/modo visual.

Não criar conflito com translation.

---

# Modos de manipulação

Adicionar modo explícito:

```text
MOVE
ROTATE
SCALE
```

O operador pode trocar pela interface inicialmente.

Isso é preferível a tentar inferir tudo automaticamente nesta versão.

Exemplo:

```text
MODE
[ MOVE ] [ ROTATE ] [ SCALE ]
```

---

# Por que usar modos

A prioridade da v0.2 é estabilidade.

Não tentar criar uma linguagem gestual “mágica” que constantemente confunda:

* mover;
* girar;
* escalar.

A interface pode evoluir depois.

---

# MOVE mode

Usar:

```text
PINCH
```

para mover.

---

# ROTATE mode

Usar:

```text
PINCH
```

e deslocamento da mão para alterar rotação.

Exemplo:

```text
deltaX → rotationY
deltaY → rotationX
```

Adicionar sensibilidade configurável.

---

# SCALE mode

Usar duas mãos.

Quando ambas estiverem detectadas:

usar distância relativa.

Não exigir pinch simultâneo se isso se mostrar instável.

Pode utilizar duas mãos abertas como gatilho da sessão.

Priorizar o comportamento que funcionar melhor empiricamente.

---

# Calibração

Criar:

# HAND CALIBRATION

Objetivo:

ajustar interação ao operador, câmera e distância.

Adicionar ação:

```text
CALIBRAR
```

Fluxo simples:

1. usuário coloca as mãos em posição confortável;
2. sistema coleta amostras por alguns segundos;
3. calcula parâmetros básicos;
4. salva configuração.

---

# Parâmetros de calibração

Pode determinar:

* pinch start threshold;
* pinch release threshold;
* smoothing;
* faixa útil de movimento;
* distância confortável entre mãos.

Não tentar personalização excessiva.

---

# Persistência

Salvar configurações de calibração localmente.

Usar o mecanismo de persistência já existente no Azriel quando apropriado.

Não criar solução paralela desnecessária.

---

# Reset de calibração

Adicionar:

```text
RESETAR CALIBRAÇÃO
```

---

# HUD

Atualizar Engineering View para mostrar:

```text
HAND TRACKING

LEFT            DETECTED
RIGHT           DETECTED

GESTURE L       OPEN
GESTURE R       PINCH

MODE            ROTATE
TRACKING FPS    28
CALIBRATION     ACTIVE
```

---

# Object Inspector

Mostrar:

```text
OBJECT
TEST-01

POSITION
X ...
Y ...
Z ...

ROTATION
X ...
Y ...
Z ...

SCALE
1.42
```

Valores devem refletir estado real do objeto.

---

# Feedback visual

Durante MOVE:

```text
CONTROL
TRANSLATE
```

Durante ROTATE:

```text
CONTROL
ROTATE
```

Durante SCALE:

```text
CONTROL
SCALE
```

---

# Bounding / Workspace

Criar limites básicos para o objeto.

Evitar:

* sair completamente da câmera;
* escala absurda;
* posições impossíveis.

Não precisa adicionar física.

---

# Perda de uma mão durante SCALE

Se uma das mãos sumir:

```text
SCALE SESSION
CANCELLED
```

Objeto permanece no último tamanho válido.

Não gerar salto.

---

# Perda de tracking durante ROTATE/MOVE

Preservar comportamento da v0.1:

```text
tracking lost
↓
interação cancelada
↓
objeto permanece onde estava
```

---

# Two-hand identity

Evitar trocar repentinamente mão esquerda por direita entre frames.

Se MediaPipe fornecer handedness e confidence, usar esses dados para estabilização.

---

# Performance

Tracking de duas mãos aumenta custo.

Monitorar FPS.

Evitar:

* processamento duplicado;
* criação excessiva de objetos por frame;
* state updates React a cada landmark individual quando desnecessário.

Utilizar refs/estruturas eficientes quando apropriado.

---

# Camera Preview

Preservar opção existente.

No debug, mostrar cores/labels diferentes para:

```text
LEFT
RIGHT
```

---

# Configurações do Engineering Core

Adicionar seção ou painel para:

```text
PINCH SENSITIVITY
ROTATION SENSITIVITY
SMOOTHING
MIN SCALE
MAX SCALE
```

Manter valores razoáveis por padrão.

Não exigir que o usuário configure tudo.

---

# Presets

Opcionalmente criar:

```text
PRECISION
BALANCED
FAST
```

apenas se isso simplificar a configuração.

Não é obrigatório.

---

# Testes

Adicionar testes para:

1. identificação de duas mãos;
2. POINT;
3. OPEN_HAND;
4. prioridade gestual;
5. Scale Session;
6. distância inicial;
7. scale factor;
8. min scale;
9. max scale;
10. smoothing de escala;
11. rotation delta;
12. rotation sensitivity;
13. perda de mão durante scale;
14. troca de modo;
15. calibração;
16. persistência da calibração;
17. reset de calibração.

Utilizar landmarks simulados.

Não depender de webcam real nos testes automatizados.

---

# Documentação

Criar:

```text
docs/engineering-core/v0.2.md
```

Documentar:

* duas mãos;
* novos gestos;
* modos;
* translation;
* rotation;
* scale;
* calibração;
* smoothing;
* limitações;
* decisões de UX.

Atualizar a documentação do Engineering Core para indicar:

```text
v0.1 — concluída
v0.2 — Spatial Manipulation
```

---

# Critérios de aceite

Engineering Core v0.2 só está concluído quando:

1. tracking de duas mãos funcionar;
2. esquerda/direita forem diferenciadas;
3. pinch da v0.1 continuar funcionando;
4. POINT funcionar de forma utilizável;
5. OPEN_HAND funcionar de forma utilizável;
6. modo MOVE funcionar;
7. modo ROTATE funcionar;
8. modo SCALE funcionar;
9. scale utilizar duas mãos;
10. escala for uniforme;
11. min/max scale funcionarem;
12. rotação X/Y funcionar;
13. smoothing funcionar;
14. troca de modo funcionar;
15. calibração existir;
16. calibração for persistida;
17. reset de calibração funcionar;
18. HUD mostrar mãos e gestos reais;
19. inspector mostrar posição;
20. inspector mostrar rotação;
21. inspector mostrar escala;
22. perda de tracking não causar saltos;
23. perda de uma mão cancelar scale com segurança;
24. performance continuar utilizável;
25. câmera continuar exclusivamente local;
26. AI Core continuar sem acesso aos frames;
27. TypeScript não apresentar erros;
28. testes passarem;
29. build funcionar;
30. Tauri iniciar normalmente;
31. demais módulos do Azriel continuarem funcionando.

---

# Teste final obrigatório

## MOVE

Selecionar:

```text
MODE = MOVE
```

Fazer pinch sobre TEST-01.

Mover a mão.

Confirmar que o objeto acompanha.

Soltar.

Confirmar posição.

---

## ROTATE

Selecionar:

```text
MODE = ROTATE
```

Fazer pinch.

Mover horizontalmente.

Confirmar rotação em Y.

Mover verticalmente.

Confirmar rotação em X.

Soltar.

Confirmar rotação preservada.

---

## SCALE

Selecionar:

```text
MODE = SCALE
```

Mostrar duas mãos.

Afastar.

Confirmar aumento.

Aproximar.

Confirmar redução.

Retirar uma mão durante a operação.

Confirmar que a escala permanece estável.

---

## CALIBRAÇÃO

Executar:

```text
CALIBRAR
```

Completar processo.

Fechar Azriel.

Abrir novamente.

Confirmar que configuração foi mantida.

---

# Resultado esperado

Ao concluir esta versão, o operador deve conseguir usar as próprias mãos como um controlador espacial básico.

Engineering Core deverá suportar:

# MOVE

# ROTATE

# SCALE

A v0.2 não precisa parecer magia.

Ela precisa ser previsível, estável e confortável.

Quando essa linguagem gestual estiver confiável, a próxima etapa poderá substituir TEST-01 por modelos 3D reais.

---

# Estado da implementação — 01/09/2026

Status: **Engineering Core v0.2 concluída com limitação conhecida de calibração**.

Foram entregues:

* tracking local de até duas mãos;
* identidade estável LEFT/RIGHT;
* gestos PINCH, POINT, OPEN_HAND e NONE;
* modos MOVE, ROTATE e SCALE;
* translation com limites de workspace;
* rotação X/Y com sensibilidade e smoothing;
* escala uniforme com limites e cancelamento seguro;
* HUD e Object Inspector com valores reais;
* configuração manual persistente no SQLite;
* testes frontend, Rust e build Tauri.

## Exceção de aceite aprovada

A calibração automática não foi incluída no encerramento funcional desta versão.

O controle permanece identificado na interface como:

```text
CALIBRAR V0.2.1
```

e fica indisponível para evitar apresentar como concluído um fluxo ainda não validado fisicamente.

Ficam adiados para **Engineering Core v0.2.1**:

* coleta guiada de amostras;
* cálculo automático por operador e câmera;
* ajuste automático de thresholds e smoothing;
* validação após reinício;
* teste físico completo da calibração.

A migration e os contratos de persistência já existentes permanecem como fundação técnica, sem alterar o escopo aceito da v0.2.
