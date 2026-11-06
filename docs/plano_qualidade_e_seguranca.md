# Plano: engenharia de qualidade e segurança do trabalho na família UTSim

Plano técnico, sem implementação, da frente de **engenharia de qualidade** e de **engenharia de segurança do trabalho** da família UTSim, pedida pelo dono em 2026-11-06 (`docs/backlog_v02.md`, seção "Engenharia de qualidade e segurança do trabalho"). O plano define público e objetivos, compara três formatos de entrega, lista o escopo de qualidade e de segurança, descreve a integração com o 3D, com os dados e com os outros apps, propõe a arquitetura e as crates novas e divide o trabalho em etapas com o modelo de IA de cada uma. Os requisitos sugeridos usam o prefixo `QS-RF-nn` e ficam só aqui até o dono aprovar.

Documentos relacionados, que este plano não repete:

- `docs/plano_lean_seis_sigma.md`, o app UTSim Seis Sigma (DMAIC, FMEA com tabela de AP própria, MSA, plano de controle, `estudo.json`, pasta de projeto com diário);
- `docs/plano_linhas_especializadas.md`, o pacote de linha (`biblioteca.ron`, `vocabulario.ron`, campos `line` e `preset`) e o Lean Office;
- `docs/plano_chegadas_e_equipes_3d.md`, tipos de item, rótulos por expressão, equipes, prioridade e preempção;
- `docs/plano_gemeo_digital_e_painel.md`, gêmeo digital, painel, CEP (E4) e capacidade (E6);
- `docs/plano_launcher.md`, distribuição pelo Drive e manifesto de produtos.

**Aviso que vale para o plano inteiro.** O UTSim apoia o estudo, o ensino e a organização das informações. Ele não substitui o profissional legalmente habilitado, não emite laudo e não certifica conformidade com nenhuma Norma Regulamentadora. As NRs aparecem neste plano só por número e tema, sem reprodução de texto. O cumprimento da legislação continua sendo responsabilidade do usuário e da empresa (seção 11).

---

## 0. Ponto de partida

### 0.1 O que já existe e será reaproveitado

| Peça | Onde | Uso nesta frente |
|---|---|---|
| Cartas de controle (I-MR, X̄-R, X̄-S, p, np, c, u), regras de Western Electric e Nelson, EWMA, CUSUM, ARL, capacidade com intervalos, Box-Cox, percentis, MSA por ANOVA e por médias e amplitudes, atributos com kappa | `crates/utsim-quality` e comando `utsim qualidade` | CEP e capacidade sobre dado simulado e real, plano de controle, eficácia da inspeção |
| Estatística geral, testes, funções especiais | `crates/utsim-stats` | base de amostragem (binomial, hipergeométrica, Poisson) e dos indicadores |
| Ajuste de distribuições | `crates/utsim-fit` | tempo entre incidentes, distribuição da característica medida |
| Rótulos sorteados na criação do item (`labels` da `Source`), `OutputRule::ByLabel` e `OutputRule::Weighted` | `crates/utsim-model/src/lib.rs` | geração de defeito e roteamento para retrabalho e refugo |
| OEE com `reject_to` e `reject_label` | `crates/utsim-model/src/maintenance.rs` | taxa de qualidade do OEE ligada ao refugo do modelo |
| Estudo de tempos homem-máquina (elementos, ciclos, tempo padrão, tolerância de fadiga) | `crates/utsim-diagram/src/tempos` e `utsim tempos` | frequência e duração de cada elemento para a ergonomia |
| Corredores, cruzamentos pedestre e veículo, parada obrigatória antes do cruzamento (`crossing_stop`), faixas no piso, folga por papel (pessoa, empilhadeira, AGV) | `crates/utsim-nav` (`aisles.rs`, `body.rs`) | zonas de risco, interação entre pessoas e empilhadeiras |
| Rastro de movimento e reprodução | `crates/utsim-model/src/replay.rs`, `movement.rs`, `utsim-diagram/src/rastro.rs` | tempo de exposição em zona de risco |
| Operador com colete refletivo e capacete, materiais nomeados por peça | `crates/utsim-assets/src/people.rs` | EPI por posto no 3D |
| FMEA, tabela de AP própria, Pareto, Ishikawa, 5 porquês, A3, plano de controle e de resposta | `crates/utsim-lss-core/src/tools` | ferramentas comuns de solução de problemas |
| Pasta com diário de operações, instantâneos, conflitos, presença, privacidade e anonimização | `crates/utsim-lss-store`, `utsim-lss-core` (`journal`, `privacidade`, `perfis`) | registros de riscos e de incidentes compartilhados pelo Drive |
| Gêmeo digital com OPC UA, MQTT e Sparkplug, gravações em arquivo | `crates/utsim-twin` | dado real de medição, contagem de refugo e sensores de presença |
| Pacote de linha e registro de linhas | `crates/utsim-linhas` | predefinições "Qualidade" e "Segurança" no editor |
| Launcher com manifesto e códigos de acesso | `crates/utsim-launcher-core` | entrada do novo produto |

### 0.2 Lacunas principais

- O `Processor` não gera defeito nem característica medida por item. Hoje o defeito só nasce como rótulo sorteado na `Source` ou por script.
- Não há estação de inspeção com eficácia de detecção, nem laço de retrabalho com limite de passagens.
- Não há custo da qualidade (o backlog já pede custos por estado, item e recurso).
- Não há registro de posto de trabalho com perigos, riscos, EPI e checklist.
- O rastro de movimento existe, mas ninguém mede tempo dentro de um polígono nem a proximidade entre pessoa e empilhadeira.
- O operador 3D tem um só conjunto fixo de EPI.

### 0.3 Princípios

- **Cálculo no Rust, interface fina.** Toda conta mora em crate pura, testada contra exemplos resolvidos à mão, como nos outros planos.
- **O posto de trabalho é a espinha.** Característica da qualidade, perigo, risco, EPI, elemento do estudo de tempos e checklist de máquina apontam para o mesmo posto, que aponta para um objeto do modelo `.ron` quando existe.
- **Origem do dado sempre visível.** Todo conjunto carrega `origem: Real | Gêmeo | Simulado`, a mesma regra do app Seis Sigma. Um indicador de segurança calculado na simulação nunca aparece como indicador real.
- **Tabelas próprias ou cadastradas pelo usuário.** Matriz de risco, escala de ergonomia, planos de amostragem e checklist de proteção de máquinas são escritos pelo projeto. Quem tem direito de uso de uma tabela de norma ou de manual cadastra a tabela por CSV, como já acontece com a tabela de AP do FMEA.
- **Custo zero.** Nenhum servidor. Trabalho em equipe por pasta compartilhada no Drive do usuário.

---

## 1. Objetivos e público

### 1.1 Objetivos

1. Simular a qualidade do processo junto com o fluxo, para estudar onde o defeito nasce, onde é detectado, quanto custa e o efeito de mudar a inspeção, o método ou a máquina.
2. Levar o CEP e a capacidade do `utsim-quality` para o dado que o próprio modelo gera e para o dado real que chega pelo gêmeo, por CSV ou pelas placas.
3. Organizar a gestão de riscos ocupacionais por posto de trabalho num fluxo parecido com o do GRO e do PGR (NR-1), com inventário de perigos, avaliação, plano de ação e acompanhamento.
4. Usar o modelo para estudar segurança antes de mudar o arranjo físico, medindo exposição em zonas de risco e conflitos entre pessoas e empilhadeiras.
5. Ligar ergonomia ao estudo de tempos, porque repetição e duração saem dos ciclos que o UTSim já mede.
6. Gerar casos para o ensino e para o jogo de problemas industriais.

### 1.2 Público

| Público | Uso principal |
|---|---|
| Professor e alunos de engenharia de produção | disciplinas de controle da qualidade, gestão da qualidade, ergonomia, higiene e segurança do trabalho, arranjo físico |
| Engenheiro de qualidade de empresa piloto | plano de controle, inspeção, CEP, 8D, custo da qualidade |
| Técnico ou engenheiro de segurança do trabalho | inventário de riscos por posto, EPI, checklist de máquinas, registro de incidentes e indicadores |
| Belts do app Seis Sigma | dados de qualidade e de risco dentro do DMAIC |
| Consultor de melhoria | estudo de cenários de layout e inspeção antes da obra |

---

## 2. Formato de entrega (decisão aberta)

### 2.1 Opções

| Opção | Como fica | A favor | Contra |
|---|---|---|---|
| **A. App novo** "UTSim Qualidade e Segurança" (egui, como o Seis Sigma) | binário próprio com duas áreas, Qualidade e Segurança, e registros permanentes | os registros de riscos e de incidentes são permanentes e não cabem num projeto DMAIC, o público de segurança não precisa do resto do Seis Sigma, telas e permissões próprias | mais um produto no launcher, risco de duplicar ferramentas do Seis Sigma |
| **B. Módulos dentro do UTSim e do Seis Sigma** | o modelo ganha defeito, inspeção, zonas e EPI, e o Seis Sigma ganha CEP contínuo, 8D, inventário de riscos e incidentes | nenhum produto novo, reaproveita a pasta com diário | o Seis Sigma é um gerenciador de projetos com começo e fim e ficaria inchado, segurança vira apêndice de um app de melhoria |
| **C. Linhas especializadas "Qualidade" e "Segurança"** | pacotes de linha no UTSim (`linhas/qualidade`, `linhas/seguranca`) com biblioteca, vocabulário, modelos prontos e relatórios | segue o padrão do Lean Office, custo de distribuição zero, ótimo para o ensino | uma linha é um domínio de processo (escritório, hospital) e qualidade e segurança atravessam todos os domínios, além disso a linha não guarda registros permanentes |

### 2.2 Recomendação

**Combinar as três em camadas** (decisão G1).

1. **No núcleo do UTSim, sem depender de formato.** Defeito por processo, característica medida, estação de inspeção, retrabalho, refugo, zonas de risco, EPI por posto e cálculo de exposição. Essas peças servem a qualquer linha, inclusive ao Lean Office, que já tem o %C&A.
2. **Predefinições como pacotes de linha transversais.** "Qualidade" e "Segurança" entram no `utsim-linhas` como pacotes que se somam à linha principal do modelo (um modelo Lean Office pode ligar o pacote Qualidade). Isso pede um campo `packs: Vec<String>` além de `line` (decisão G2).
3. **Um app novo com duas áreas** para o que é registro permanente e gestão (posto de trabalho, plano de controle, inspeção, custo da qualidade, 8D, inventário de riscos, ergonomia, checklist, incidentes e indicadores). As ferramentas que já existem no Seis Sigma (FMEA, Pareto, Ishikawa, 5 porquês, A3, plano de controle) saem de `utsim-lss-core::tools` para uma crate comum, usada pelos dois apps (seção 8).

Nome provisório **UTSim Qualidade e Segurança**, produto `qualidade-seguranca` no launcher, prefixo de crates `utsim-qs-*`. A alternativa de dois apps separados fica registrada na decisão G1.

---

## 3. Escopo de qualidade

### 3.1 Defeito gerado por processo (núcleo)

Bloco opcional `quality:` em `Processor`, `Combiner` e `Separator`.

| Campo | O que faz |
|---|---|
| `defects` | lista de tipos de defeito, cada um com probabilidade base e rótulo que recebe no item (`defeito = "risco_pintura"`) |
| `factors` | multiplicadores da probabilidade por condição observável no núcleo, como itens desde o último setup, primeira peça depois de quebra, habilidade do operador, turno, rótulo do item |
| `characteristics` | característica medida por item (`diametro`), com distribuição, deriva por item ou por hora e salto depois de setup, guardada como rótulo numérico |
| `spec` | limites de especificação da característica, que definem o defeito por medição |

O item leva os rótulos adiante. Uma etapa seguinte pode piorar ou mascarar o defeito. Os valores medidos de cada característica vão para a saída da replicação em formato longo (item, objeto, característica, instante, valor), lido direto pelo `utsim-quality`.

### 3.2 Inspeção, retrabalho e refugo (núcleo e pacote Qualidade)

| Predefinição | Base | Comportamento |
|---|---|---|
| Estação de inspeção | `Processor` com bloco `inspection:` | tempo de inspeção, amostragem (100 %, um a cada n, por lote com plano da seção 3.3), probabilidade de detectar cada tipo de defeito, probabilidade de rejeitar item bom (alarme falso), saídas aprovado, retrabalho e refugo |
| Inspeção por medição | idem | aplica a variação do sistema de medição (desvio da MSA) ao valor real e compara com a especificação, o que gera erro de classificação perto dos limites |
| Retrabalho | `Processor` com laço de volta | limite de passagens por item (rótulo `passagens`), defeito pode sumir ou não, item que estoura o limite vai para o refugo |
| Refugo | `Sink` com papel `refugo` | conta por tipo de defeito, entra no OEE pelo `reject_to` existente |
| Cliente | `Sink` com papel `cliente` | conta defeitos que escaparam (falha externa) |

Indicadores calculados por replicação, com intervalo de confiança: rendimento de primeira passagem por etapa, rendimento acumulado, % de refugo e de retrabalho, defeitos por unidade, DPMO, defeitos escapados, eficácia de cada inspeção, carga das estações de retrabalho e efeito da inspeção no gargalo.

### 3.3 Plano de controle, plano de inspeção e plano de amostragem (app)

- **Plano de controle** por posto e característica, com método de medição, frequência, tamanho da amostra, carta usada, plano de reação e ligação com a linha correspondente do FMEA. É a mesma estrutura do plano de controle do Seis Sigma, que passa para a crate comum.
- **Plano de inspeção** por posto, com o que se inspeciona, como, quando e por quem, ligado às estações de inspeção do modelo.
- **Plano de amostragem por atributos**, simples e duplo, calculado pelo próprio programa a partir do risco do produtor e do consumidor e dos níveis de qualidade escolhidos, com curva característica de operação, qualidade média de saída e inspeção total média. O cálculo usa as distribuições binomial, hipergeométrica e Poisson. As tabelas das normas de amostragem não são embutidas. Quem tem direito de uso cadastra os planos por CSV.
- **Teste do plano no modelo.** O plano escolhido vira o bloco `inspection:` de uma estação e o simulador mostra o efeito sobre escape, custo e fluxo.

### 3.4 CEP e capacidade

- Sobre dado simulado, as cartas e os índices saem da característica gerada na seção 3.1, por replicação e no conjunto das replicações, usando os limites a partir do modelo já previstos no `utsim-quality`.
- Sobre dado real, CSV, gêmeo ou placas (seção 6), em fase I e fase II.
- **Comparação real e simulado.** Mesma característica nas duas origens, lado a lado, com resíduo pela EWMA de resíduos que já existe. Serve para validar o modelo e para achar deriva.
- Sinal de carta abre cartão de reação no plano de controle e, se o usuário quiser, um 8D (seção 3.7) ou um projeto no Seis Sigma.

### 3.5 Custo da qualidade

Quatro categorias usuais (prevenção, avaliação, falha interna, falha externa). Os valores vêm de duas fontes.

| Fonte | Exemplo |
|---|---|
| Contagem do modelo vezes custo unitário | horas da inspeção, peças refugadas, horas de retrabalho, defeitos escapados |
| Lançamento manual no app | treinamento, calibração, garantia, devoluções |

Relatório com total por categoria, por posto e por tipo de defeito, e comparação entre cenários com números aleatórios comuns. Depende do item de custos do backlog. Enquanto ele não existir, o custo unitário fica no bloco `quality:` do objeto.

### 3.6 Pareto e Ishikawa

Pareto por tipo de defeito, por posto, por custo e por turno, gerado direto da replicação ou do dado real. Ishikawa ligado ao tipo de defeito, com as causas que o modelo conhece (fatores da seção 3.1) sugeridas como ramos. As duas ferramentas são as do Seis Sigma, movidas para a crate comum.

### 3.7 Solução de problemas com 8D e A3

- **8D** com as oito disciplinas descritas em palavras próprias, contenção com prazo, causa raiz ligada ao Ishikawa e aos 5 porquês, ação corretiva verificada por dado (carta ou estudo no modelo), prevenção de recorrência que atualiza FMEA e plano de controle.
- **A3** reaproveitado da crate comum.
- Um 8D pode virar projeto DMAIC no Seis Sigma quando a causa não aparece (seção 7.1).

### 3.8 Ligação com FMEA, MSA e DMAIC

| Peça | Ligação |
|---|---|
| FMEA | o modo de falha aponta para o tipo de defeito do modelo e para a linha do plano de controle, a ocorrência pode ser conferida com a taxa simulada ou real, a detecção com a eficácia da estação de inspeção |
| MSA | o resultado do Gage R&R alimenta a variação da inspeção por medição (seção 3.2) e o plano de controle mostra se o sistema de medição foi aprovado |
| DMAIC | o projeto do Seis Sigma lê o plano de controle, as cartas e o custo da qualidade como dados do Medir e do Controlar |

A tabela de AP usada continua a própria do UTSim ou a cadastrada pela empresa, nunca uma cópia da tabela do manual.

---

## 4. Escopo de segurança do trabalho

### 4.1 Inventário de perigos e riscos por posto (fluxo no estilo GRO e PGR)

Fluxo em etapas, inspirado na lógica do gerenciamento de riscos ocupacionais da NR-1, sem reproduzir o texto da norma.

1. **Identificar** o posto, as atividades (vindas dos elementos do estudo de tempos quando há modelo) e as pessoas expostas.
2. **Levantar perigos** por grupo (físico, químico, biológico, ergonômico, acidente e mecânico), com fonte, circunstância e possíveis lesões.
3. **Avaliar** cada risco pela matriz da seção 4.2, com as medidas de controle existentes.
4. **Planejar ações** pela hierarquia de controle (eliminar, substituir, controle de engenharia, controle administrativo, EPI), com responsável e prazo.
5. **Acompanhar** a eficácia, com reavaliação e histórico.

O app gera um relatório de inventário e plano de ação por posto e por área. O relatório traz no rodapé o aviso de que não é o documento legal do PGR e de que a responsabilidade técnica e legal é da empresa.

### 4.2 Matriz de risco própria

- Matriz configurável de 3x3 a 5x5, probabilidade por severidade, com níveis e cores definidos pelo projeto.
- Escalas descritas em palavras próprias, com critérios de exemplo editáveis.
- Regras de decisão por nível (aceitável, ação planejada, ação imediata) editáveis pela empresa.
- Importação CSV da matriz da empresa, que passa a valer no registro inteiro.
- Testes conferem monotonia da matriz (mais severidade ou mais probabilidade nunca baixa o nível), como foi feito com a tabela de AP.

### 4.3 Ergonomia ligada ao estudo de tempos

- O estudo de tempos (`utsim tempos`) fornece, por elemento, frequência por hora, duração média, tempo por ciclo e fração do turno.
- O usuário marca, por elemento, postura (tronco, braços, pescoço, pernas), força ou carga manipulada, pega e vibração, em escalas próprias de poucos níveis.
- **Pontuação própria do UTSim.** Combina postura, carga, repetição e duração numa nota por elemento e por posto, com regras monótonas e pesos documentados. Inspirada na lógica pública dos métodos observacionais, sem copiar tabelas de nenhum método publicado. A empresa pode cadastrar o método que já usa por CSV.
- **Comparação de métodos.** O mesmo estudo antes e depois de uma mudança mostra a variação da nota junto com a variação do tempo padrão.
- **No modelo.** Um posto com nota alta pode ganhar pausa obrigatória ou rodízio de pessoas (recursos que o núcleo já tem em turnos e equipes), e o efeito sobre a produção aparece no cenário.
- A análise ergonômica exigida pela NR-17, quando aplicável, continua sendo um trabalho do profissional responsável. O app organiza dados e não emite a análise.

### 4.4 EPI por posto

- Cadastro de EPI do app (tipo, uso, CA informado pelo usuário, validade, troca) e EPI exigido por posto e por perigo, com referência à NR-6 só por número e tema.
- O posto do modelo ganha `ppe: Vec<String>` (capacete, óculos, protetor auricular, luvas, botina, colete, máscara, avental).
- No 3D a pessoa veste o EPI do posto em que está (seção 5).
- Relatório de EPI por pessoa a partir das equipes e dos postos que cada pessoa visita na simulação.

### 4.5 Interação entre pessoas e empilhadeiras e zonas de risco

- **Zonas de risco** como polígonos do layout, com tipo (circulação de veículo, raio de máquina, área de carga suspensa, ruído, calor), nível de risco e EPI exigido para entrar. Ficam no modelo e são desenhadas com as faixas de piso do `utsim-nav`.
- **Regras de circulação** que já existem no `utsim-nav` (corredores por tipo, cruzamentos, parada obrigatória antes do cruzamento, larguras mínimas) são conferidas e listadas como medidas de controle no inventário.
- **Conferência do layout** com avisos para corredor de pedestre abaixo da largura mínima do perfil, cruzamento sem parada, posto dentro de zona de veículo, rota de pessoa que atravessa zona de veículo fora de cruzamento.
- A NR-11 é citada como tema (transporte e movimentação de materiais) sem conferência automática de conformidade.

### 4.6 Simulação da exposição

Calculado sobre o rastro de movimento de cada replicação.

| Medida | Como sai |
|---|---|
| Tempo em zona de risco | tempo de cada pessoa dentro de cada polígono, por turno e por replicação, com intervalo |
| Entradas sem EPI exigido | entrada de pessoa cujo posto atual não prevê o EPI da zona |
| Proximidade pessoa e empilhadeira | instantes em que a distância fica abaixo de um limiar configurável, contados como **quase-conflito simulado** |
| Cruzamentos | passagens por cruzamento e esperas da parada obrigatória |
| Mapa de calor | densidade de presença e de quase-conflitos no piso |

Os quase-conflitos simulados servem para comparar layouts e regras de circulação. Eles não estimam a probabilidade de um acidente real e o relatório diz isso.

### 4.7 Registro de quase-acidentes e incidentes e indicadores

- Registro com data, hora, posto, zona, tipo (quase-acidente, incidente sem afastamento, com afastamento, dano material), descrição, causas, ações e ligação com o risco do inventário.
- Pessoas identificadas por código. Nome e dados de saúde só em pasta restrita, com anonimização reaproveitada do módulo `privacidade` do Seis Sigma, por causa da LGPD (risco R5).
- Indicadores com base configurável em horas-homem de exposição, como taxa de frequência, taxa de gravidade, dias sem acidente, razão entre quase-acidentes e incidentes e tendência mensal com carta de controle para contagens (c ou u do `utsim-quality`).
- A tabela de dias debitados de norma técnica não é embutida. A empresa cadastra a dela.
- Pareto de causas e de postos e mapa de incidentes sobre o layout.

### 4.8 Checklist de proteção de máquinas

- Checklist próprio por máquina, agrupado por tema (proteções fixas e móveis, intertravamento, parada de emergência, comandos, sinalização, manutenção e bloqueio, capacitação), com itens escritos pelo projeto e referência à NR-12 só por número e tema.
- Resposta conforme, não conforme ou não se aplica, com foto anexa e ação.
- Checklists editáveis e importáveis por CSV, para a empresa usar o roteiro do seu responsável técnico.
- Item não conforme vira risco no inventário do posto.

---

## 5. Integração com o 3D

| Recurso | Como aparece no cliente Godot |
|---|---|
| EPI por posto | operador de `utsim-assets` ganha peças opcionais por EPI (capacete, óculos, abafador, luvas, máscara, avental), geradas por código, ligadas pelos nomes dos nós e materiais que já existem, e trocadas quando a pessoa chega a um posto |
| Falta de EPI | contorno vermelho na pessoa que está numa zona sem o EPI exigido, ligado ou desligado na camada de segurança |
| Zonas de risco | polígonos semitransparentes no piso, com cor pelo nível da matriz, editáveis como os corredores (`aisle_edit.rs`) |
| Mapa de calor | textura no piso com exposição ou quase-conflitos, por replicação |
| Quase-conflito | marcador no ponto e no instante, com salto da reprodução até ele |
| Defeito no item | item com cor ou marca pelo rótulo de defeito, refugo em caçamba, retrabalho com seta de volta |
| Estação de inspeção | ícone de lupa e contador de aprovados, retrabalhados e refugados |
| CEP no posto | pequeno painel acima do posto com a carta da característica e o último sinal |
| Painéis egui | listas de riscos do posto, EPI exigido e checklist ao clicar no posto |

Os ativos começam gerados por código e CC0, sem custo, na mesma regra da decisão L6 das linhas. Uniformes e EPI mais detalhados seguem a tarefa de ambiente e pessoas do backlog.

---

## 6. Integração de dados

| Fonte | O que traz | Como entra |
|---|---|---|
| CSV | medições, contagens de defeito, incidentes, horas-homem, inventário de riscos, checklists | formato longo documentado, com mapeamento de colunas na importação |
| Gêmeo digital (`utsim-twin`, OPC UA, MQTT, Sparkplug) | característica medida por peça, contador de refugo, presença em zona, posição de empilhadeira | tag do gêmeo apontada pelo plano de controle ou pela zona, com gravação em JSONL |
| Protocolo futuro das placas (ESP32, Arduino e outras, backlog) | sensores de baixo custo como presença, ruído, temperatura, vibração, contagem de peças | mensagem com tipo de grandeza, unidade, posto e instante, a ser desenhada com o dono junto do protocolo geral |
| Modelo UTSim | característica simulada, rastro, exposição, custos | biblioteca direta (os apps são Rust), `estudo.json` quando atravessa pessoa ou tempo |

Todo conjunto guarda origem, hash e unidade. Medições de ambiente vindas das placas servem para estudo e ensino e não substituem medição com instrumento calibrado exigida pela legislação (risco R3).

---

## 7. Integração com os outros apps

### 7.1 UTSim Seis Sigma

- Ferramentas comuns (FMEA, AP, Pareto, Ishikawa, 5 porquês, A3, plano de controle) numa só crate, usada pelos dois apps.
- Um 8D, um sinal de carta ou um risco alto podem abrir uma ideia na carteira do Seis Sigma, com os dados anexados.
- O projeto DMAIC lê plano de controle, cartas, custo da qualidade e indicadores de segurança como conjuntos de dados com origem.
- O plano de controle aprovado no Controlar de um projeto volta para o app como plano permanente do posto.

### 7.2 App de ensino e jogo de problemas industriais

Pacotes de conteúdo, cada um com modelo `.ron`, dados com efeito plantado, enunciado, perguntas e gabarito com tolerância.

| Pacote sugerido | Problema |
|---|---|
| Refugo escondido | defeito que nasce depois do setup e só é visto no cliente |
| Inspeção que atrapalha | inspeção 100 % vira gargalo, aluno escolhe plano de amostragem |
| Medição ruim | Gage R&R alto faz a inspeção rejeitar peça boa |
| Cruzamento perigoso | layout com rota de pedestre cortando corredor de empilhadeira |
| Posto pesado | estudo de tempos com nota de ergonomia alta e escolha entre rodízio e mudança de método |
| Custo da qualidade | comparar prevenção e falha |

O formato do pacote é decidido no plano do app de ensino. Este plano só pede que ele aceite os blocos `quality:`, `inspection:`, zonas e EPI.

### 7.3 Linhas especializadas

- Pacotes transversais "Qualidade" e "Segurança" combináveis com qualquer linha (decisão G2).
- No Lean Office, o %C&A vira um caso particular do defeito por processo e a verificação vira estação de inspeção. Ergonomia de escritório entra pelo mesmo estudo de elementos.
- Nas linhas de logística e manufatura, as zonas de risco e a interação com empilhadeiras são as peças mais usadas.

### 7.4 Launcher

Nova entrada `qualidade-seguranca` no manifesto, canal e código de acesso próprios, tag `qs-v0.1.0`, e atalho "abrir no UTSim" pelo mesmo mecanismo do Seis Sigma.

---

## 8. Arquitetura e crates novas

### 8.1 Crates

| Crate | Tipo | Conteúdo | Depende de |
|---|---|---|---|
| `utsim-quality` (existe, cresce) | pura | planos de amostragem por atributos, curva característica de operação, AOQ e ATI, eficácia de inspeção a partir da MSA | `utsim-stats` |
| `utsim-ferramentas` (nova, extraída) | pura | FMEA, AP, Pareto, Ishikawa, 5 porquês, A3, plano de controle e de reação, 8D | `utsim-stats` |
| `utsim-qs-core` (nova) | pura | posto de trabalho, inventário de perigos, matriz de risco, ergonomia, EPI, checklist, incidentes e indicadores, custo da qualidade, plano de inspeção, regras e migrações do formato | `utsim-ferramentas`, `utsim-quality` |
| `utsim-exposicao` (nova, ou módulo de `utsim-model`) | pura | tempo em polígono, proximidade entre pessoas e veículos, mapa de calor, sobre o rastro | `utsim-model`, `utsim-nav` |
| `utsim-qs-store` (nova) ou `utsim-lss-store` generalizada | nativa | pasta com diário, pastas restritas, anexos por hash | `utsim-qs-core` |
| `utsim-qs-app` (nova, binário `utsim-qualidade-seguranca`) | egui `=0.36.2` | telas das duas áreas, ponte com o simulador em fio de trabalho, PDF e `.xlsx` | as acima, `utsim-model`, `utsim-diagram` |

Mudanças no núcleo, em `utsim-model` e `utsim-objects`, são os blocos `quality:` e `inspection:`, o papel `refugo` e `cliente` do `Sink`, `zones:` no modelo, `ppe:` no objeto e a saída longa de características. Todos opcionais, com migração de formato e modelos antigos abrindo como hoje.

### 8.2 Regras de dependência

- Nenhuma crate comum depende de crate de app.
- `utsim-lss-core` passa a usar `utsim-ferramentas`, com migração do formato do projeto Seis Sigma para manter os FMEA já salvos.
- A extração das ferramentas acontece antes de qualquer tela nova, para não haver duas versões (decisão G3).

### 8.3 Armazenamento

Uma pasta de **unidade** (fábrica ou laboratório) com o cadastro de postos, o inventário de riscos, os planos e os checklists, e pastas restritas para incidentes com dados pessoais. Mesmo diário de operações do Seis Sigma, compartilhado pelo Drive.

---

## 9. Roteiro em etapas

Horas Opus 5.5 estimadas. Opus para formato, estatística, regras e geometria. Sonnet para telas, relatórios, ativos e testes a partir de plano. Haiku para tarefas mecânicas.

### 9.1 MVP

O MVP entrega um estudo completo em sala ou na empresa piloto. Um modelo com defeito por processo, inspeção, retrabalho e refugo, com relatório de qualidade e Pareto. Zonas de risco e EPI por posto no modelo e no 3D, com tempo de exposição e quase-conflitos simulados. No app, cadastro de postos, inventário de riscos com matriz própria e plano de ação. Ficam fora do MVP o custo da qualidade completo, os planos de amostragem, o 8D, a ergonomia, o checklist de máquinas, os incidentes e as placas.

### 9.2 Etapas

| Etapa | Conteúdo | Horas | Modelo | Depende de | Critério de aceite |
|---|---|---|---|---|---|
| **QS0 Desenho do formato** (MVP) | blocos `quality:`, `inspection:`, `zones:`, `ppe:`, papéis do `Sink`, campo `packs`, migração | 5–7 | Opus | decisões G1, G2 | documento de formato aprovado, modelos antigos passam na migração sem mudança de resultado |
| **QS1 Defeito e característica no núcleo** (MVP) | sorteio do defeito com fatores, característica com deriva e salto, rótulos numéricos, saída longa | 12–16 | Opus | QS0 | taxa de defeito simulada dentro do intervalo da probabilidade teórica em 30 replicações, mesma semente dá mesma saída |
| **QS2 Inspeção, retrabalho e refugo** (MVP) | estação com amostragem simples, detecção e alarme falso, inspeção por medição, laço com limite, `Sink` refugo e cliente, OEE | 12–16 | Opus 8–10, Sonnet 4–6 | QS1 | casos resolvidos à mão para escape e rendimento acumulado batem com a simulação, item nunca passa do limite de retrabalho |
| **QS3 Relatório de qualidade** (MVP) | rendimentos, DPMO, Pareto por defeito e posto, cartas e capacidade da característica simulada no relatório HTML e no `utsim qualidade` | 8–12 | Sonnet | QS2 | relatório abre sem rede, valores conferidos com `utsim-quality` sobre o mesmo CSV |
| **QS4 Pacotes de linha Qualidade e Segurança** (MVP) | predefinições de inspeção, retrabalho, refugo, cliente, zona de risco, vocabulário, dois modelos prontos | 6–9 | Sonnet | QS2, `utsim-linhas` | predefinição colocada no editor gera `.ron` válido e legível sem o pacote |
| **QS5 Zonas, EPI e exposição** (MVP) | `utsim-exposicao`, tempo em polígono, proximidade, entradas sem EPI, mapa de calor, avisos de conferência do layout | 14–18 | Opus | QS0, `utsim-nav` | tempo em zona conferido com trajetória de geometria conhecida, contagem de proximidade estável com passo de amostragem menor |
| **QS6 Camada de segurança e qualidade no 3D** (MVP) | peças de EPI geradas por código, troca no posto, zonas no piso e edição, mapa de calor, marcadores de quase-conflito, cor do item com defeito, ícone da inspeção | 14–20 | Sonnet 12–17, Haiku 1–2 | QS5, QS2 | pessoa troca EPI ao entrar no posto na reprodução, regressão visual com capturas comparadas aprovada |
| **QS7 Extração de `utsim-ferramentas`** (MVP) | mover ferramentas do `utsim-lss-core`, migração do projeto Seis Sigma | 6–9 | Opus 3–4, Sonnet 3–5 | decisão G3 | todos os testes do Seis Sigma passam, projeto antigo abre igual |
| **QS8 Fundação do app** (MVP) | `utsim-qs-core`, `utsim-qs-store` ou store generalizada, binário egui, pasta de unidade, cadastro de postos ligado ao modelo, entrada no launcher | 22–30 | Opus 10–14, Sonnet 12–16 | QS7 | duas pessoas editam a mesma pasta pelo Drive sem perder operação, posto ligado ao objeto do modelo por id |
| **QS9 Inventário de riscos e matriz** (MVP) | fluxo em cinco etapas, matriz própria configurável e importável, hierarquia de controle, plano de ação, relatório PDF com aviso legal | 16–22 | Opus 4–6, Sonnet 12–16 | QS8 | matriz monótona em todas as combinações, relatório traz o aviso, CSV da empresa substitui a matriz |
| *Subtotal do MVP* | | *115–159* | *Opus ≈ 60–80, Sonnet ≈ 53–76, Haiku ≈ 1–2* | | |
| **QS10 Custo da qualidade** | categorias, custo unitário no modelo, lançamentos manuais, comparação de cenários | 10–14 | Opus 3–4, Sonnet 7–10 | QS3, custos do backlog | soma por categoria confere com contagens vezes custos em caso de teste |
| **QS11 Planos de controle, inspeção e amostragem** | planos permanentes por posto, amostragem simples e dupla, curva OC, AOQ, ATI, plano aplicado ao modelo | 16–22 | Opus 10–13, Sonnet 6–9 | QS8, QS2 | probabilidades conferidas com cálculo direto da binomial e da hipergeométrica, plano gerado cumpre os dois riscos pedidos |
| **QS12 CEP real e comparado** | importação CSV, fase I e II, comparação com o simulado, cartão de reação | 10–14 | Sonnet 8–11, Opus 2–3 | QS11 | mesmos limites que o `utsim qualidade` para o mesmo arquivo |
| **QS13 8D e ligação com o Seis Sigma** | 8D, envio para a carteira, retorno do plano de controle | 10–14 | Sonnet 8–11, Opus 2–3 | QS7, QS8 | 8D aberto vira ideia no Seis Sigma com dados anexados e hash conferido |
| **QS14 Ergonomia** | marcação por elemento sobre o `utsim tempos`, pontuação própria monótona, comparação de métodos, importação de método da empresa | 14–20 | Opus 8–11, Sonnet 6–9 | QS8, `utsim-diagram` | nota nunca baixa quando postura, carga, repetição ou duração pioram, comparação usa o mesmo par de estudos do `tempos` |
| **QS15 Checklist de máquinas e EPI** | checklists próprios e importáveis, foto anexa, não conformidade vira risco, cadastro de EPI com validade | 10–14 | Sonnet 9–12, Haiku 1–2 | QS9 | item não conforme aparece no inventário do posto com origem |
| **QS16 Incidentes e indicadores** | registro, pasta restrita, anonimização, taxas com base configurável, carta de contagens, Pareto, mapa no layout | 14–20 | Opus 4–6, Sonnet 10–14 | QS8 | taxas conferidas em caso resolvido à mão, nome nunca sai da pasta restrita nos relatórios |
| **QS17 Dados do gêmeo e das placas** | tags no plano de controle e nas zonas, presença real em zona, mensagem das placas | 10–16 | Opus 5–8, Sonnet 5–8 | QS12, protocolo das placas | gravação de teste reproduz o mesmo indicador que o CSV equivalente |
| **QS18 Pacotes de conteúdo para o ensino** | os seis pacotes da seção 7.2 com dados, enunciado e gabarito | 12–18 | Sonnet 10–15, Opus 2–3 | QS6, plano do app de ensino | cada gabarito confere com a simulação dentro da tolerância |
| *Total depois do MVP* | | *106–152* | | | |
| **Total geral** | | **221–311** | | | |

### 9.3 Ordem sugerida

1. QS0, depois QS1 e QS5 em paralelo (arquivos diferentes, núcleo do item e geometria do rastro).
2. QS2 e QS7 em paralelo, depois QS3, QS4 e QS6.
3. QS8 e QS9 fecham o MVP.
4. Depois do MVP, a ordem pelo público inicial (decisão G4). Ensino favorece QS14, QS18 e QS10. Empresa piloto favorece QS11, QS12, QS16 e QS15.

### 9.4 Testes

- Casos resolvidos à mão para cada fórmula, com tolerância declarada.
- Testes de monotonia para matriz de risco, AP e ergonomia, cobrindo todas as combinações.
- Testes de geometria com trajetórias e polígonos conhecidos.
- Reprodutibilidade por semente em todos os indicadores simulados.
- Regressão visual do 3D com capturas comparadas.
- Conferência de que nenhum relatório de segurança sai sem o aviso legal.

### 9.5 Requisitos sugeridos

| Id | Requisito |
|---|---|
| QS-RF-01 | O processo pode gerar defeitos e características medidas por item, com fatores observáveis no núcleo |
| QS-RF-02 | A estação de inspeção aplica amostragem, detecção, alarme falso e variação de medição |
| QS-RF-03 | O retrabalho respeita um limite de passagens por item |
| QS-RF-04 | O relatório mostra rendimentos, DPMO, escape e Pareto por replicação, com intervalo |
| QS-RF-05 | O modelo guarda zonas de risco e EPI por posto e o 3D os mostra |
| QS-RF-06 | A simulação mede tempo em zona, entradas sem EPI e quase-conflitos entre pessoas e veículos |
| QS-RF-07 | O app mantém cadastro de postos ligado ao modelo e inventário de riscos com matriz configurável |
| QS-RF-08 | Todo relatório de segurança traz o aviso de responsabilidade legal do usuário |
| QS-RF-09 | Tabelas de norma ou de manual só entram por cadastro do usuário |
| QS-RF-10 | Dados pessoais de incidentes ficam em pasta restrita e podem ser anonimizados |

---

## 10. Decisões do dono

**Aceitas pelo dono em 2026-11-06.** Todas as propostas desta seção valem como decididas. Onde não havia proposta fica o padrão abaixo, que o dono pode trocar a qualquer momento.

- **G1, revista pelo dono no mesmo dia:** "Qualidade deve ser uma, Segurança do Trabalho, outra". Em vez de um app com duas áreas, são **dois apps**: **UTSim Qualidade** (binário `utsim-qualidade`, crate `utsim-qualidade-app`, produto `qualidade` no launcher) e **UTSim Segurança do Trabalho** (binário `utsim-seguranca`, crate `utsim-seguranca-app`, produto `seguranca`). O que os dois usam fica em crates comuns: `utsim-ferramentas` (ferramentas extraídas do Seis Sigma) e `utsim-posto` (posto de trabalho e pasta de unidade). O cálculo de cada app fica em `utsim-qualidade-core` e `utsim-seguranca-core`. As camadas do núcleo e dos pacotes de linha não mudam. Onde o texto abaixo fala em "app com duas áreas", leia os dois apps.
- **G4:** ensino primeiro, empresa piloto depois.
- **S2:** matriz 5x5.
- **S4:** limiar padrão de 1,5 m, sem depender da velocidade no MVP. É revisto na QS5 com o dono.

O MVP (QS0 a QS9) começa numa sessão própria.

### Gerais

1. **G1 Formato.** Proposta: camadas da seção 2.2 (núcleo, pacotes de linha transversais, um app com duas áreas). Alternativas: dois apps separados, tudo dentro do Seis Sigma, só linhas.
2. **G2 Pacotes transversais.** Proposta: campo `packs` no modelo, ao lado de `line`, para combinar Qualidade e Segurança com qualquer linha.
3. **G3 Ferramentas comuns.** Proposta: extrair FMEA, AP, Pareto, Ishikawa, 5 porquês, A3 e plano de controle para `utsim-ferramentas` antes das telas novas.
4. **G4 Público inicial.** Ensino, empresa piloto ou os dois. Define a ordem depois do MVP.
5. **G5 Nome.** Provisório "UTSim Qualidade e Segurança", prefixo `utsim-qs-*`.
6. **G6 Armazenamento.** Proposta: generalizar `utsim-lss-store` em vez de criar `utsim-qs-store`.

### Qualidade

7. **Q1 Defeito por processo no MVP.** Proposta: probabilidade base com fatores de setup, quebra, habilidade e rótulo. Alternativa: só probabilidade fixa no começo.
8. **Q2 Característica medida.** Proposta: guardada como rótulo numérico do item com saída longa. Alternativa: só na saída, sem viajar no item.
9. **Q3 Planos de amostragem.** Proposta: calcular os planos pela curva OC, sem tabelas de norma, com cadastro por CSV para quem tem direito.
10. **Q4 Custo da qualidade.** Esperar o item de custos do backlog ou começar com custo unitário no bloco `quality:`.
11. **Q5 8D.** Proposta: no app novo, com envio para o Seis Sigma. Alternativa: só no Seis Sigma.
12. **Q6 Origem do plano de controle.** Proposta: um só plano por posto, compartilhado entre o app e os projetos do Seis Sigma.

### Segurança

13. **S1 Fluxo de riscos.** Proposta: cinco etapas inspiradas no GRO, com relatório que não se apresenta como PGR.
14. **S2 Matriz padrão.** Tamanho padrão 5x5 ou 4x4, sempre editável e importável.
15. **S3 Ergonomia.** Proposta: pontuação própria monótona sobre o `utsim tempos`, com importação do método da empresa. Alternativa: só registro qualitativo.
16. **S4 Limiar de quase-conflito.** Distância padrão e se depende da velocidade do veículo.
17. **S5 Incidentes no MVP.** Proposta: fora do MVP, por causa da LGPD e da pasta restrita. Alternativa: registro simples sem dados pessoais já no MVP.
18. **S6 Checklist de máquinas.** Proposta: checklist próprio por tema, com importação do roteiro do responsável técnico.
19. **S7 Sensores de ambiente das placas.** Aceitar ruído e temperatura das placas como dado de estudo, com aviso de que não substituem medição legal.

### Integração

20. **X1 Pacotes de conteúdo.** Confirmar os seis pacotes da seção 7.2 como primeiros casos do app de ensino.
21. **X2 Lean Office.** Tratar o %C&A como caso particular do defeito por processo, com migração das predefinições.

---

## 11. Riscos

| Id | Risco | Mitigação |
|---|---|---|
| R1 | **Uso do app como prova de conformidade legal.** Um inventário ou checklist gerado pelo UTSim pode ser tomado como cumprimento de NR | aviso fixo em telas e relatórios de que o app não emite laudo, não substitui o profissional habilitado e não garante conformidade, e de que o cumprimento da legislação é do usuário e da empresa. Nenhum texto de norma reproduzido, só número e tema |
| R2 | **Leitura errada da simulação de segurança.** Quase-conflito simulado tomado como previsão de acidente | rótulo "simulado" em todo indicador, nota no relatório, comparação só entre cenários do mesmo modelo |
| R3 | **Dado de sensor barato tratado como medição oficial** | origem sempre visível, aviso nas grandezas ambientais, campo de instrumento e calibração |
| R4 | **Direito autoral de tabelas.** Tentação de embutir tabela de amostragem, de dias debitados, de método ergonômico ou a AP do manual | só tabelas próprias ou cadastradas pelo usuário, revisão de cada tabela nova contra essa regra, como foi feito com a AP |
| R5 | **Dados pessoais e de saúde (LGPD)** nos incidentes | pasta restrita, código no lugar do nome, anonimização, perfis de acesso do Seis Sigma |
| R6 | **Duplicação com o Seis Sigma** | extração para `utsim-ferramentas` antes das telas (G3) |
| R7 | **Crescimento do núcleo** com blocos novos | todos opcionais, migração testada, custo zero quando ausentes, medido em `docs/desempenho.md` |
| R8 | **Escopo amplo demais para um dono** | MVP fechado na seção 9.1, etapas depois do MVP ordenadas pelo público (G4) |
| R9 | **Ergonomia com nota única simplificar demais** | nota sempre acompanhada dos fatores, escala documentada, aviso de que não é a análise exigida pela NR-17 |
| R10 | **Ativos de EPI pouco realistas** confundirem o aluno | começar com peças simples e legíveis, detalhe depois pela tarefa de ambiente e pessoas do backlog |
