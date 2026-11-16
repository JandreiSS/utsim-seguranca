# Formato dos blocos de qualidade e de segurança do trabalho

Etapa QS0 de `docs/plano_qualidade_e_seguranca.md`. Descreve os campos que o modelo `.ron` ganhou para a frente de qualidade e de segurança do trabalho. Todos são opcionais. O formato continua na versão 2 (`FORMAT_VERSION`), porque nenhum campo antigo mudou de sentido. Um modelo sem estes blocos abre e roda exatamente como antes, e os golden não mudam.

O código mora em `crates/utsim-model/src/quality.rs` (tipos, validação e o gancho de entrada) e em `crates/utsim-model/src/lib.rs` (campos do objeto e do modelo).

## 1. Campos novos

| Onde | Campo | Tipo | Lido pela simulação |
|---|---|---|---|
| objeto | `quality` | `QualityDef` | sim |
| objeto | `inspection` | `InspectionDef` | sim |
| objeto | `rework` | `ReworkDef` | sim |
| objeto `Sink` | `sink_role` | `Scrap` (ou `Refugo`) e `Customer` (ou `Cliente`) | não, só relatórios |
| objeto | `ppe` | lista de nomes de EPI | não, só exposição, app e 3D |
| modelo | `zones` | lista de `ZoneDef` | não, só exposição, app e 3D |
| modelo | `packs` | lista de nomes de pacote | não, só interface |

`quality`, `inspection` e `rework` valem em `Processor`, `Combiner`, `Separator` e `Queue`. Um objeto pode ter mais de um. A ordem de aplicação, quando o item entra no objeto, é: o script `on_entry` do usuário, o sorteio de `quality`, o `rework` e por fim a `inspection`.

## 2. Rótulos que o item carrega

| Rótulo | Valor | Quem escreve |
|---|---|---|
| `defeito.<tipo>` | 1 presente, 0 corrigido | `quality` (sorteio ou característica fora da especificação), `rework` |
| `defeitos` | quantos defeitos estão presentes | `quality`, `rework` |
| `<característica>` | valor medido | `quality` |
| `<característica>@t` | instante da medição | `quality` |
| `<característica>@lsl`, `@usl` | limites, para a inspeção medir em outro objeto | `quality` |
| `passagens` | passagens pelo retrabalho | `rework` |
| `inspecao` (ou o nome de `inspection.label`) | porta de saída escolhida | `inspection` |

A saída longa das características (item, objeto, característica, instante, valor) sai do registro por item (`Model::run_item_log`) lendo esses rótulos.

## 3. `quality`

```ron
quality: Some((
    defects: [
        (name: "risco", probability: 0.02, factors: [
            Cycle(every: 50, first: 5, times: 3.0),
            Label(label: "turno", equals: 3, times: 1.5),
        ]),
    ],
    characteristics: [
        (name: "diametro", value: Normal(mean: 10.0, sd: 0.02),
         drift: 0.0005, cycle: Some(200), shift: Some(Normal(mean: 0.0, sd: 0.01)),
         lsl: Some(9.95), usl: Some(10.05)),
    ],
)),
```

- **Defeito.** Probabilidade base por item, multiplicada pelos fatores e limitada a 1. `Cycle` multiplica nos `first` primeiros itens de cada ciclo de `every` itens do objeto (peças logo depois do setup ou da troca de ferramenta). `Label` multiplica quando o item tem o rótulo numérico igual a `equals` (habilidade, turno, lote de material). Um número é sempre sorteado por tipo de defeito, para que mudar a probabilidade de um tipo não desloque o sorteio dos outros.
- **Característica.** Valor sorteado da distribuição, mais o deslocamento do ciclo (`shift`, sorteado no primeiro item de cada ciclo) e mais a deriva (`drift` vezes a posição do item no ciclo). Fora dos limites, o item ganha `defeito.<característica>`.
- Custos (etapa QS10, nunca lidos pela simulação): `unit_cost` é o valor que o processo agrega a cada peça, perdido quando ela é refugada, e `escape_cost` é o custo de cada defeito do processo que chega ao cliente. A inspeção tem `cost` por peça inspecionada e o retrabalho tem `cost` por passagem. O manual, no capítulo "Defeitos, inspeção e retrabalho", descreve o relatório de custos.

## 4. `inspection` e saídas

```ron
output: ByLabel("inspecao"),
inspection: Some((
    every: 1,             // 1 = 100 %, n = um a cada n
    detect: 0.9,          // probabilidade de detectar um defeito presente
    detect_by: {"risco": 0.6},
    false_reject: 0.01,   // alarme falso em item bom
    measure: [(characteristic: "diametro", sigma: 0.004)],
    on_reject: Rework,    // ou Scrap
    max_rework: Some(2),
)),
```

| `on_reject` | `max_rework` | Porta 1 | Porta 2 | Porta 3 |
|---|---|---|---|---|
| `Rework` | ausente | aprovado | retrabalho | |
| `Rework` | `Some(n)` | aprovado | retrabalho | refugo, depois de `n` passagens |
| `Scrap` | qualquer | aprovado | refugo | |

As portas seguem a ordem das ligações que saem do objeto. A validação exige `output: ByLabel(<label>)` e ligações em número suficiente. Item não amostrado passa como aprovado. Na inspeção por medição, o valor comparado com os limites é o valor real mais um erro normal de desvio `sigma` (o desvio do sistema de medição vindo da MSA), o que gera erro de classificação perto dos limites. O defeito de uma característica medida só é visto pela medição, nunca pelo `detect`.

## 5. `rework`

```ron
rework: Some((fix: 0.95, fix_by: {"risco": 0.8})),
```

Soma 1 em `passagens` e corrige cada defeito presente com a probabilidade `fix` (ou a do tipo). Uma característica corrigida volta ao meio dos limites. O item volta para a inspeção pela ligação comum, e é a inspeção que manda para o refugo quem já usou `max_rework` passagens, por isso nenhum item passa do limite.

## 6. `sink_role`, `ppe`, `zones` e `packs`

```ron
(name: "refugo", kind: Sink, sink_role: Some(Scrap)),
(name: "expedicao", kind: Sink, sink_role: Some(Customer)),
(name: "prensa", kind: Processor(..), ppe: ["oculos", "luvas", "protetor_auricular"]),

zones: [
    (name: "corredor_empilhadeira", kind: Vehicle, level: Some(4),
     polygon: [(0, 4), (30, 4), (30, 7), (0, 7)], ppe: ["colete"]),
],
packs: ["qualidade", "seguranca"],
```

- Os nomes de EPI que o 3D sabe vestir são `capacete`, `oculos`, `protetor_auricular`, `luvas`, `botina`, `colete`, `mascara` e `avental` (`quality::PPE_NAMES`). Outros nomes são guardados e listados, mas não desenhados.
- Tipos de zona: `Vehicle` (`Veiculo`), `Machine` (`Maquina`), `SuspendedLoad` (`CargaSuspensa`), `Noise` (`Ruido`), `Heat` (`Calor`) e `Other` (`Outro`). `level` é o nível da matriz de risco, de 1 em diante, usado na cor.
- `packs` soma pacotes transversais à linha do modelo (decisão G2). A simulação nunca lê.

## 7. Números aleatórios

O gancho de qualidade de cada objeto usa um fluxo próprio, `1 << 43` mais o índice do objeto, separado do fluxo do processo. Ligar ou mudar um bloco de qualidade não muda os tempos sorteados pelo objeto, e cenários comparados com os mesmos números aleatórios continuam pareados. A mesma semente dá a mesma saída.

## 8. Migração

Nenhuma. Os campos novos têm valor padrão ausente e não são gravados quando ausentes, então um modelo antigo lido e gravado de novo fica igual. O teste `quality_blocks_absent_keep_old_models` confere que todos os exemplos de `examples/` passam pela leitura sem ganhar campo novo.

## 9. Métricas no relatório da simulação

Para que experimentos, otimização e as missões do UTSim Ensino leiam a qualidade e a exposição como qualquer outra métrica (`RunReport::metrics`), o relatório ganha chaves só nos modelos que usam os blocos novos.

| Chave | Onde | O que é |
|---|---|---|
| `<saída>.pecas_defeituosas` | `Sink` com `sink_role` | peças que chegaram com algum defeito presente |
| `<saída>.pct_defeituosas` | idem | a fração delas sobre as entradas (no cliente, o escape) |
| `<saída>.retrabalhadas` | idem | peças que passaram pelo retrabalho |
| `<saída>.defeitos.<tipo>` | idem | defeitos presentes por tipo |
| `<recurso>.quase_conflitos` | recurso móvel, modelo com `zones` | quase-conflitos simulados de que participou |
| `<pessoa>.sem_epi` | pessoa, modelo com `zones` | entradas em zona sem o EPI pedido |
| `<pessoa>.tempo_zona.<zona>` e `<pessoa>.entradas_zona.<zona>` | idem | tempo e entradas em cada zona |
| `<pessoa>.tempo_sob_carga` e `<pessoa>.entradas_sob_carga` | pessoa, modelo com pontes rolantes | tempo e entradas sob carga suspensa (todas as pontes) |
| `<ponte>.tempo_sob_carga` e `<ponte>.entradas_sob_carga` | ponte rolante, modelo com pessoas | tempo e entradas de pessoas sob a carga dela |

As contagens das saídas valem na janela de estatística, como `input`. A exposição usa as opções padrão (limiar de 1,5 m). Num modelo com zonas, ou com pontes rolantes e pessoas, o rastro é gravado para esse cálculo, o que custa memória mas não muda nenhum número da simulação. Na exposição (`Exposure::under_load`, `exposicao.sob_carga.tempo` e `exposicao.sob_carga.entradas`), o tempo sob carga vem por pessoa e ponte.

## 10. Blocos das abas do nó

Pela direção do dono de 2026-11-08 (`docs/plano_integracao_familia.md`, seção 3), o nó guarda também os dados que antes ficavam só na pasta de unidade. A simulação não lê nenhum dos dois blocos.

- `quality.control`: linhas do plano de controle das características medidas no processo (`characteristic`, `method`, `msa_ok`, `frequency`, `sample_size`, `chart`, `reaction`, `fmea`). A característica precisa existir em `quality.characteristics`.
- `safety`: `hazards` (perigos do posto com `id` único no nó, `group`, `description`, `source`, `circumstance`, `injuries`, `controls`, `probability` e `severity` de 0 a 4 na matriz padrão, `origin`) e `ergonomics` (marcações por elemento do estudo de tempos, com `actor`, `element`, postura de 0 a 3 em `trunk`, `arms`, `neck` e `legs`, `load_kg`, `grip` e `vibration` de 0 a 2).

```ron
(name: "prensa", kind: Processor(process: Const(40.0)),
 ppe: ["oculos", "protetor_auricular"],
 safety: Some((
     hazards: [(id: "h1", group: Accident, description: "Prensagem da mão", probability: 1, severity: 4)],
     ergonomics: [(actor: "ana", element: "carregar prensa", trunk: 2, arms: 1, load_kg: 12.0, grip: 1)],
 ))),
```
