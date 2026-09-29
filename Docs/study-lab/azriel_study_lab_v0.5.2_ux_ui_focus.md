# AZRIEL STUDY LAB v0.5.2 — UX/UI Focus Refinement
## Roadmap View + Focus View — Prompt para Codex

## CONTEXTO

O Study Lab v0.5.1 está funcional e validado.

A estrutura de dados atual é rica e deve ser preservada:

- Roadmaps
- Stages
- Topics
- Activities
- learningObjective
- learningMethod
- instructions
- resources
- deliverable
- completionCriteria
- reflectionPrompt
- estimatedMinutes
- StudySession / Pomodoro
- Notes / Cadernos
- Library
- Review
- AI Study Tools
- Knowledge Nodes

O problema atual NÃO é falta de informação.

O problema é que a interface apresenta informação demais simultaneamente.

Hoje a tela de Roadmaps mistura:

- seleção de roadmap;
- roadmap inteiro;
- progresso;
- etapas;
- tópicos;
- atividades;
- metadata da atividade;
- objetivo;
- método;
- instruções;
- recursos;
- entrega;
- critério;
- reflexão;
- notas;
- controles administrativos.

Isso gera alta densidade visual e prejudica o uso do Study Lab como ambiente de concentração.

Esta versão deve alterar UX/UI, não o modelo pedagógico.

---

# 1. OBJETIVO

Implementar:

`STUDY LAB v0.5.2 — UX/UI FOCUS REFINEMENT`

Princípio:

> A tela de Roadmap serve para escolher o que estudar.
> A tela de Focus serve para estudar.

Criar dois estados de experiência claramente diferentes:

```text
ROADMAP VIEW
    ↓
selecionar atividade
    ↓
preview compacto
    ↓
INICIAR ESTUDO
    ↓
FOCUS VIEW
```

Não remover dados.

Não simplificar o backend.

Não alterar JSON/schema apenas para resolver layout.

---

# 2. REGRA DE DESIGN

Aplicar progressive disclosure.

Mostrar agora apenas o que é necessário para a decisão atual.

Informação secundária permanece disponível sob demanda.

Evitar:

- todos os painéis abertos;
- metadata repetida;
- estados duplicados;
- bordas em excesso;
- múltiplos CTAs concorrentes;
- controles administrativos durante estudo.

---

# 3. INSPEÇÃO ANTES DE CODIFICAR

Antes de alterar:

1. revisar componente atual de Estudos;
2. localizar Roadmap View;
3. localizar Activity View;
4. localizar StudySession/Pomodoro;
5. localizar Notes/Library/AI panels;
6. localizar navegação interna;
7. identificar componentes reutilizáveis;
8. identificar responsive behavior;
9. preservar design tokens do AZRIEL;
10. registrar plano curto.

Não reescrever backend.

---

# 4. ROADMAP VIEW — FINALIDADE

Roadmap View responde somente:

- qual roadmap estou vendo?
- onde estou nele?
- qual é o próximo tópico/atividade?
- o que posso estudar agora?

Ela NÃO é a tela para consumir todos os detalhes pedagógicos da atividade.

---

# 5. REMOVER COLUNA FIXA DE ROADMAPS

A lista lateral permanente `SEUS ROADMAPS` ocupa espaço demais.

Substituir preferencialmente por seletor compacto:

```text
ROADMAP

Empreendedorismo Tecnológico    ▾
```

Ao abrir:

```text
Engenharia de Tecnologia
English for Technology
Empreendedorismo Tecnológico
```

Mostrar progresso pequeno junto ao nome quando útil.

Se a arquitetura visual existente justificar outra solução igualmente compacta, usar julgamento.

Objetivo: recuperar espaço horizontal.

---

# 6. CABEÇALHO DO ROADMAP

Manter:

- nome;
- descrição curta;
- progresso;
- CONTINUAR ESTUDO.

Reduzir metadata visual.

Exemplo:

```text
EMPREENDEDORISMO TECNOLÓGICO
Evidence-Driven Venture Path

0% · 0/15 atividades

[ CONTINUAR ESTUDO ]
```

Não repetir `PLANEJADO`, contagens e progresso em vários lugares.

---

# 7. AÇÕES ADMINISTRATIVAS

Hoje ações como:

- Importar JSON
- Exportar JSON
- Novo Roadmap
- Editar
- Excluir

ficam visualmente presentes durante estudo.

Agrupar ações menos frequentes em menu de gerenciamento:

```text
[ ⋯ ]

Novo Roadmap
Importar JSON
Exportar JSON
Editar Roadmap
Excluir Roadmap
```

`+ NOVO ROADMAP` pode permanecer visível somente se o design atual realmente justificar.

Prioridade é reduzir ruído.

---

# 8. STAGES

Stage deve ser compacto.

Exemplo:

```text
01  Baseline Empresarial e Sistema de Decisão       0/2
    Diagnosticar decisões reais e substituir opinião por evidência.
```

Expandir apenas Stage atual/selecionado por padrão.

Demais stages podem permanecer recolhidos.

Não abrir todas as atividades de todas as etapas simultaneamente.

---

# 9. TOPICS

Topic deve mostrar:

- número;
- título;
- state;
- progresso simples.

Exemplo:

```text
1.1  Retrospectiva empresarial baseada em evidência
     NÃO INICIADO                              0/1
```

Evitar múltiplas barras de progresso quando número simples resolver.

---

# 10. ACTIVITY PREVIEW

Ao selecionar Activity, NÃO mostrar imediatamente a ficha completa.

Mostrar preview compacto.

Exemplo:

```text
RETROSPECTIVA EMPRESARIAL

Identificar padrões pessoais de decisão,
risco e execução.

REFLECTION · ~150 min · CHECKPOINT

[ INICIAR ESTUDO ]
```

Opcionalmente:

```text
1 recurso
1 entrega
```

Sem mostrar todo conteúdo.

---

# 11. REMOVER METADATA REDUNDANTE

Não repetir simultaneamente:

- activityType no topo e em card;
- PENDENTE em múltiplos lugares;
- status em tabela e header;
- SESSION AVAILABLE;
- conclusão vazia;
- checkpoint duas vezes;
- progresso duplicado.

Uma informação deve ter um lugar visual principal.

---

# 12. INICIAR ESTUDO

`INICIAR ESTUDO` entra no Focus View.

Se StudySession/Pomodoro exigir confirmação/configuração, fazer transição simples.

Não abrir Focus View automaticamente apenas porque Activity foi selecionada.

Seleção ≠ início do estudo.

---

# 13. FOCUS VIEW — FINALIDADE

Focus View é a sala de estudo.

Ao entrar:

- esconder árvore completa do roadmap;
- esconder seletor de roadmaps;
- esconder ações administrativas;
- reduzir navegação secundária;
- dar prioridade ao conteúdo da atividade.

Deve ser possível voltar ao Roadmap View.

---

# 14. FOCUS VIEW — HEADER

Exemplo:

```text
← EMPREENDEDORISMO TECNOLÓGICO

RETROSPECTIVA EMPRESARIAL BASEADA EM EVIDÊNCIA

Identificar padrões pessoais de decisão,
risco e execução.
```

Header pode mostrar discretamente:

```text
REFLECTION · CHECKPOINT · ~150 min
```

Não usar grande painel de metadata.

---

# 15. FOCUS VIEW — CONTEÚDO PRIMÁRIO

Visível inicialmente:

## OBJETIVO

`learningObjective`

## O QUE FAZER

`instructions`

## RECURSOS

`resources`

Essas três áreas são o núcleo da execução.

---

# 16. CONTEÚDO SECUNDÁRIO RECOLHIDO

Usar accordions/sections recolhidas:

```text
▸ MÉTODO
▸ ENTREGA E CRITÉRIO
▸ REFLEXÃO
▸ NOTAS RELACIONADAS
```

Abrir conforme momento da atividade.

Não abrir tudo por default.

---

# 17. MÉTODO

Ao expandir:

```text
MÉTODO
PROBLEM SOLVING

Formule hipóteses antes de alterar o sistema.
```

Não ocupar card grande quando recolhido.

---

# 18. ENTREGA E CRITÉRIO

Agrupar:

```text
ENTREGA
...

CRITÉRIO DE CONCLUSÃO
...
```

Faz sentido estarem juntos porque ambos tratam do resultado da atividade.

Pode permanecer recolhido durante execução inicial.

---

# 19. REFLEXÃO

Reflection aparece como seção própria, recolhida.

Pode ganhar destaque quando atividade estiver perto de ser concluída.

Não forçar preenchimento.

A ação existente de registrar reflexão em Note continua disponível quando aplicável.

---

# 20. RECURSOS

Recursos devem parecer materiais de estudo, não metadata de banco.

Exemplo:

```text
RECURSOS

Testing Business Ideas
Strategyzer · Book · EN

[ ABRIR ]   [ + BIBLIOTECA ]
```

Evitar mostrar IDs internos.

`required` pode aparecer discretamente:

```text
OBRIGATÓRIO
```

ou

```text
OPCIONAL
```

---

# 21. POMODORO / STUDY SESSION

Focus View deve integrar StudySession sem dominar a tela.

Exemplo compacto:

```text
FOCO
25:00

[ INICIAR ]
```

Durante execução:

```text
FOCO  18:42
[ PAUSAR ]
```

Não criar painel gigante.

---

# 22. ESTIMATED MINUTES

Mostrar como referência:

```text
~150 min estimados
```

Não competir visualmente com Pomodoro.

Tempo estimado e tempo real continuam semanticamente separados.

---

# 23. NOTES

Notes não devem ficar permanentemente abertas.

No Focus View:

```text
▸ NOTAS RELACIONADAS (2)
```

Ao expandir:

- abrir nota;
- criar nota;
- nota de entrega;
- reflexão, quando aplicável.

Não embutir editor inteiro por default.

---

# 24. AI STUDY TOOLS

AI também não deve dominar a tela.

Usar ação compacta:

```text
[ AZRIEL AI ▾ ]
```

ou painel recolhível.

Ações:

- EXPLICAR
- RESUMIR
- ME TESTAR
- GERAR CARDS

Somente quando usuário pedir.

Não abrir painel AI automaticamente.

---

# 25. LIBRARY

`+ BIBLIOTECA` continua nos Resources.

Biblioteca completa não deve aparecer dentro do Focus View.

Material associado pode ser aberto sob demanda.

---

# 26. REVIEW

Review não precisa estar visível durante atividade comum.

Study Cards gerados/criados entram no sistema normal de Review.

Evitar misturar fila de revisão com execução da Activity.

---

# 27. TOP NAVIGATION

Preservar:

- HOJE
- ROADMAPS
- CONHECIMENTOS
- CADERNOS
- BIBLIOTECA
- REVISÃO
- HISTÓRICO

Mas durante Focus View avaliar modo reduzido/discreto se a arquitetura permitir.

Não criar navegação nova desnecessária.

---

# 28. FULLSCREEN / DISTRACTION-FREE

Se simples de implementar com layout existente, permitir:

`MODO FOCO`

que reduz ainda mais chrome/sidebar/header.

Não é obrigatório para aceite da v0.5.2.

Não criar fullscreen complexo se exigir grande refatoração.

---

# 29. RESPONSIVIDADE

Roadmap View deve funcionar sem depender de três colunas fixas.

Evitar layout atual:

```text
roadmap list | roadmap | activity details
```

Preferir:

```text
roadmap selector
        ↓
roadmap content
        +
activity preview
```

Em telas grandes, preview pode continuar lateral se permanecer compacto.

Focus View deve priorizar largura de leitura confortável, não preencher 100% com texto.

---

# 30. LARGURA DE LEITURA

Textos longos de instructions/objective/criteria devem ter largura controlada.

Evitar linhas extremamente longas em monitor widescreen.

Usar max-width coerente com design existente.

---

# 31. BORDAS

O AZRIEL usa bordas como parte da identidade.

Manter identidade, mas reduzir "caixas dentro de caixas".

Nem toda seção precisa de card completo.

Usar:

- spacing;
- headings;
- divisores;
- accordions;

antes de adicionar nova borda.

---

# 32. CORES

Não redesenhar paleta.

Preservar cyan/dark e estados atuais.

Amarelo/checkpoint deve continuar sem dominar a página inteira.

---

# 33. TIPOGRAFIA

Preservar identidade.

Melhorar hierarquia por:

- tamanho;
- peso;
- spacing;
- contraste;

não adicionando novas fontes.

---

# 34. EMPTY STATES

Preservar estados vazios claros.

Focus View nunca deve mostrar cards vazios de:

- resources;
- notes;
- reflection;
- deliverable.

Se dado não existe, seção não aparece.

---

# 35. PROGRESSIVE DISCLOSURE

Regra central:

```text
ROADMAP VIEW
mostra decisão de navegação

FOCUS VIEW
mostra execução

ACCORDION
mostra detalhe quando necessário
```

Evitar qualquer retorno ao padrão "mostrar tudo porque existe".

---

# 36. NÃO ALTERAR DADOS

Não remover:

- learningObjective;
- learningMethod;
- instructions;
- resources;
- deliverable;
- completionCriteria;
- reflectionPrompt.

A mudança é apresentação.

---

# 37. NÃO ALTERAR ROADMAP JSON

Import/export continua compatível com v0.5.1.

Nenhuma mudança no schema é necessária apenas para UX.

---

# 38. NÃO QUEBRAR STUDY SESSION

Roadmap View → Focus View → voltar não pode:

- perder timer;
- duplicar StudySession;
- resetar atividade;
- perder Notes.

---

# 39. DEEP LINK / STATE

Se arquitetura atual já suporta route/state para Activity:

preservar.

Refresh/reopen deve recuperar contexto de forma coerente.

Não criar router complexo apenas para Focus View.

---

# 40. KEYBOARD / ESC

Se modal/panel for usado:

ESC fecha painel secundário, não cancela StudySession.

Não implementar atalhos complexos nesta versão.

---

# 41. TESTES — ROADMAP VIEW

Validar:

- seletor de roadmap;
- troca entre os 3 roadmaps;
- stages recolhidos;
- stage atual expandido;
- topic selection;
- activity preview;
- CONTINUAR ESTUDO;
- menu administrativo;
- progresso correto.

---

# 42. TESTES — FOCUS VIEW

Validar:

- entrar;
- voltar;
- objetivo;
- instruções;
- resources;
- accordions;
- método;
- entrega/critério;
- reflexão;
- Notes;
- AI panel;
- Pomodoro;
- Activity completion.

---

# 43. TESTES — SESSÃO ATIVA

Fluxo:

```text
Roadmap View
→ selecionar Activity
→ INICIAR ESTUDO
→ Focus View
→ iniciar Pomodoro
→ abrir Resource
→ criar Note
→ usar AI
→ recolher painel
→ voltar Roadmap
→ retornar Focus
```

Timer e contexto permanecem.

---

# 44. TESTES — REGRESSÃO

Garantir:

- import/export JSON;
- Roadmap edit;
- Activity edit;
- Library;
- Review;
- Notes;
- AI Study Tools;
- Knowledge;
- History;
- Today;
- StudySession.

---

# 45. CRITÉRIOS DE ACEITE

v0.5.2 concluída quando:

1. Roadmap View não usa três áreas massivas simultâneas;
2. lista fixa de Roadmaps é removida ou compactada;
3. seletor de Roadmap é simples;
4. ações administrativas deixam de dominar header;
5. stages podem permanecer recolhidos;
6. Activity selection mostra preview compacto;
7. preview não mostra ficha completa;
8. INICIAR ESTUDO entra em Focus View;
9. Focus View prioriza Activity;
10. árvore completa fica fora do Focus View;
11. objetivo/instruções/resources ficam visíveis;
12. método fica recolhível;
13. entrega+critério ficam recolhíveis;
14. reflexão fica recolhível;
15. Notes ficam recolhíveis;
16. AI fica sob demanda;
17. Pomodoro é compacto;
18. metadata redundante é removida;
19. recursos parecem materiais, não registros de banco;
20. nenhuma informação pedagógica é apagada;
21. schema/JSON v0.5.1 permanece compatível;
22. StudySession não é perdida/duplicada;
23. identidade visual AZRIEL permanece;
24. telas largas têm leitura confortável;
25. regressões principais passam;
26. build passa;
27. Tauri inicia;
28. documentação existe.

---

# 46. DOCUMENTAÇÃO

Criar:

`docs/study-lab/v0.5.2-ux-ui.md`

ou caminho equivalente.

Documentar:

- problema anterior;
- Roadmap View;
- Activity Preview;
- Focus View;
- progressive disclosure;
- comportamento de StudySession;
- accordions;
- ações administrativas;
- decisões responsivas;
- testes;
- limitações.

---

# 47. ENTREGA CODEX

Ao concluir, resumir:

- componentes alterados;
- componentes novos;
- Roadmap View;
- Focus View;
- Activity Preview;
- comportamento de accordions;
- integração Pomodoro/Notes/AI;
- regressões;
- testes;
- build/Tauri;
- limitações.

---

# RESULTADO ESPERADO

Antes:

```text
ROADMAP LIST
+
ROADMAP COMPLETO
+
ACTIVITY COMPLETA
+
TODAS AS FERRAMENTAS
=
ALTA DENSIDADE
```

Depois:

```text
ROADMAP VIEW
Escolher onde estudar
        ↓
ACTIVITY PREVIEW
Entender rapidamente
        ↓
INICIAR ESTUDO
        ↓
FOCUS VIEW
Executar uma coisa por vez
        ↓
DETALHES SOB DEMANDA
```

O Study Lab deve parecer uma sala de estudos, não um painel administrativo.

Preservar profundidade do sistema e reduzir profundidade visual simultânea.
