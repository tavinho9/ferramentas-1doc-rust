//! Funções puras: datas, critérios de etapa, formulário da busca, log
//! (porta de tests/test_funcoes.py).

use chrono::NaiveDate;
use ferramentas1doc::datas;
use ferramentas1doc::saida::{TelaEmMemoria, Terminal};
use ferramentas1doc::texto;

fn d(a: i32, m: u32, dia: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(a, m, dia).unwrap()
}

const HOJE: (i32, u32, u32) = (2026, 1, 15);

fn hoje() -> NaiveDate {
    d(HOJE.0, HOJE.1, HOJE.2)
}

#[test]
fn interpretar_data() {
    let casos = [
        ("15/01/2026", Some(d(2026, 1, 15))),
        ("15/01/26", Some(d(2026, 1, 15))),
        ("15-01-2026", Some(d(2026, 1, 15))),
        ("15012026", Some(d(2026, 1, 15))),
        ("10/01", Some(d(2026, 1, 10))),
        ("20/12", Some(d(2025, 12, 20))), // sem ano e no futuro: ano anterior
        ("31/02/2026", None),
        ("abc", None),
    ];
    for (txt, esperado) in casos {
        assert_eq!(datas::interpretar_data(txt, hoje()), esperado, "{txt}");
    }
}

#[test]
fn faltantes_ler_data() {
    let casos = [
        ("", None),
        ("  ", None),
        ("hoje", Some(hoje())),
        ("Ontem", Some(d(2026, 1, 14))),
        ("-30", Some(d(2025, 12, 16))),
        ("- 7", Some(d(2026, 1, 8))),
        ("01/01/2025", Some(d(2025, 1, 1))),
        ("1/1/25", Some(d(2025, 1, 1))),
        ("10/01", Some(d(2026, 1, 10))),
        ("20/12", Some(d(2025, 12, 20))),
    ];
    for (txt, esperado) in casos {
        assert_eq!(datas::ler_data(txt, hoje()), Ok(esperado), "{txt}");
    }
}

#[test]
fn faltantes_ler_data_invalida() {
    for txt in ["31/02/2026", "abc", "-99999999999", "2026"] {
        assert!(datas::ler_data(txt, hoje()).is_err(), "{txt}");
    }
}

#[test]
fn faltantes_ler_periodo() {
    assert_eq!(datas::ler_periodo("", "", hoje()), Ok((None, None)));
    assert_eq!(
        datas::ler_periodo("01/12/2025", "ontem", hoje()),
        Ok((Some(d(2025, 12, 1)), Some(d(2026, 1, 14))))
    );
    assert_eq!(
        datas::ler_periodo("", "31/12/2020", hoje()),
        Ok((None, Some(d(2020, 12, 31))))
    );
    assert!(
        datas::ler_periodo("", "32/01/2026", hoje())
            .unwrap_err()
            .contains("Data final não reconhecida")
    );
    assert!(
        datas::ler_periodo("16/01/2026", "", hoje())
            .unwrap_err()
            .contains("ainda não chegou")
    );
    assert!(
        datas::ler_periodo("10/01/2026", "09/01/2026", hoje())
            .unwrap_err()
            .contains("depois da final")
    );
}

#[test]
fn ler_numero() {
    assert_eq!(
        texto::ler_numero("Proc. Administrativo 31.632/2026"),
        Some(("31.632/2026".into(), "31632".into(), "2026".into()))
    );
    assert_eq!(
        texto::ler_numero("031632/2026"),
        Some(("31.632/2026".into(), "31632".into(), "2026".into()))
    );
    assert_eq!(texto::ler_numero("5712/2025").unwrap().0, "5.712/2025");
    assert_eq!(
        texto::ler_numero("1234567/2025").unwrap().0,
        "1.234.567/2025"
    );
    assert_eq!(texto::ler_numero("sem número"), None);
}

#[test]
fn norm_e_sem_acento() {
    assert_eq!(texto::norm("  Nº Processo "), "n processo");
    assert_eq!(texto::norm("Número-do Processo"), "numero do processo");
    assert_eq!(
        texto::normtxt("  Validação  de\nRequisição "),
        "validacao de requisicao"
    );
    assert_eq!(texto::limpo(" a \n\t b\u{a0}c "), "a b c");
}

#[test]
fn terminal_reescreve_linha_com_retorno_de_carro() {
    let mut t = Terminal::default();
    let mut tela = TelaEmMemoria::default();
    for pedaco in [
        "Lendo caixas\n",
        "  Em aberto: página 1/3",
        "\r  Em aberto: página 2/3",
        "\r",
        "  Em aberto: página 3/3, 25 processos",
        "\n",
        "fim\r\n",
    ] {
        t.alimentar(pedaco, &mut tela);
    }
    let linhas: Vec<&str> = tela.texto.split('\n').collect();
    assert_eq!(
        linhas,
        [
            "Lendo caixas",
            "  Em aberto: página 3/3, 25 processos",
            "fim",
            ""
        ]
    );
}

#[test]
fn etapa_dentro_24h() {
    use ferramentas1doc::processos_do_dia as pdd;
    for (txt, dentro) in [
        ("Esta aqui 1 hora", true),
        ("Aqui há alguns segundos", true),
        ("Está aqui 45 minutos", true),
        ("Está aqui 3 dias", false),
        ("Está aqui 1 semana", false),
        ("", false),
        ("???", false),
    ] {
        assert_eq!(pdd::etapa_dentro_24h(txt), dentro, "{txt}");
    }
}

#[test]
fn kanban_ignora_copia_do_modelo_de_despacho() {
    use ferramentas1doc::processos_do_dia as pdd;
    let html = concat!(
        r#"<div id="emissao_principal"><div class="kanban_estagios" data-current="true">"#,
        r#"<div class="kanban_titulo"><p>Validação de Requisição</p></div>"#,
        r#"<div class="kanban_itens"><p class="kanban_item">Esta aqui 3 dias</p></div></div></div>"#,
        r#"<div id="parent_wa"><div class="kanban_estagios" data-current="true">"#,
        r#"<div class="kanban_titulo"><p>Validação de Requisição</p></div>"#,
        r#"<div class="kanban_itens"><p class="kanban_item">Aqui há alguns segundos</p></div></div></div>"#
    );
    assert_eq!(
        pdd::ler_etapa_atual(html),
        Some(("Validação de Requisição".into(), "Esta aqui 3 dias".into()))
    );
}

#[test]
fn rotulo_etapa() {
    use ferramentas1doc::processos_do_dia as pdd;
    assert_eq!(
        pdd::rotulo_etapa("Etapa Final", "Abertura de Procedimento Licitatório", false),
        "Etapa Final — Abertura"
    );
    assert_eq!(
        pdd::rotulo_etapa("", "Elaboração de Documentação Preparatória", true),
        "Elaboração"
    );
    assert_eq!(
        pdd::rotulo_etapa("Etapa Final", "Outra fase", false),
        "Elaboração"
    );
}

#[test]
fn descricao_do_objeto() {
    use ferramentas1doc::processos_do_dia as pdd;
    let campo =
        |dentro: &str| format!(r#"<div id="campo_adicional_descricao_objeto">{dentro}</div>"#);
    assert_eq!(
        pdd::ler_descricao_objeto(&campo(
            r#"<div class="cp"><span data-valor="Compra de &lt;br&gt;papel">Objeto<span>*</span>:</span><small>x</small></div>"#
        )),
        "Compra de\npapel"
    );
    assert_eq!(
        pdd::ler_descricao_objeto(&campo(
            r#"<div class="cp"><span>Objeto*:</span><small>linha 1<br>linha   2</small></div>"#
        )),
        "linha 1\nlinha 2"
    );
    assert_eq!(
        pdd::ler_descricao_objeto(&campo("<p>Objeto: um</p><p>dois</p>")),
        "um\ndois"
    );
    assert_eq!(pdd::ler_descricao_objeto("<div id='outro'>x</div>"), "");
}

mod busca_argv_testes {
    use chrono::NaiveDate;
    use ferramentas1doc::busca_argv::{self as ba, Formulario};

    fn agora() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 6)
            .unwrap()
            .and_hms_opt(9, 5, 0)
            .unwrap()
    }

    fn form(caixas: &[&str]) -> Formulario {
        Formulario {
            caixas: caixas.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn montar_argv_completo() {
        let mut f = form(&["entrada", "arquivo"])
            .com("numeros", "31.632/2026 5712/2025")
            .com("etapa", "Pesquisa de Mercado; Consulta Orçamentária")
            .com("fase", "Abertura")
            .com("antes_de", "Pesquisa de Mercado")
            .com("emissao_de", "01/10/2026")
            .com("dias_etapa_min", "15")
            .com("objeto", "medicamento")
            .com(
                "extras",
                r#"--setor-etapa CPPFI --numeros-arquivo "C:\Users\joao\Área de Trabalho\x.xlsx""#,
            )
            .marcar("refazer");
        f.prioridade = vec!["Urgente".into(), "Alta".into()];
        let argv =
            ba::montar_argv(&f, Some(std::path::Path::new("C:\\saida")), agora(), false).unwrap();
        assert_eq!(
            argv[..4],
            [
                "--caixas",
                "entrada,arquivo",
                "--numeros",
                "31.632/2026,5712/2025"
            ]
        );
        assert_eq!(argv.iter().filter(|a| *a == "--etapa").count(), 2);
        let i = argv.iter().position(|a| a == "--prioridade").unwrap();
        assert_eq!(argv[i..i + 2], ["--prioridade", "Urgente,Alta"]);
        assert_eq!(
            argv[argv.len() - 2..],
            [
                "--numeros-arquivo",
                r"C:\Users\joao\Área de Trabalho\x.xlsx"
            ]
        );
        assert_eq!(ba::validar(&argv), None);
    }

    #[test]
    fn montar_argv_todas_as_caixas_e_todas_prioridades() {
        let mut f = form(&ba::CAIXAS);
        f.prioridade = ba::PRIORIDADES.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            ba::montar_argv(&f, None, agora(), false).unwrap(),
            ["--caixas", "todas"]
        );
    }

    #[test]
    fn busca_aceita_os_formatos_da_instrucao_de_datas() {
        for exemplo in ba::EXEMPLOS_DATA {
            let argv: Vec<String> = ["--emissao-de", exemplo, "--mudanca-ate", exemplo]
                .iter()
                .map(|s| s.to_string())
                .collect();
            assert_eq!(ba::validar(&argv), None, "{exemplo}");
        }
    }

    #[test]
    fn instrucao_de_datas_da_busca() {
        for formato in ["DD/MM/AAAA", "AAAA-MM-DD", "hoje", "ontem", "-N"] {
            assert!(ba::DICA_DATAS.contains(formato));
        }
        // a instrução pede o ano completo: sem ele, a busca recusa
        assert!(
            ba::validar(&["--emissao-de".into(), "05/11".into()])
                .unwrap()
                .contains("data inválida")
        );
    }

    fn v(x: &[&str]) -> Vec<String> {
        x.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ordenar_etapas() {
        let base = v(&ba::ETAPAS_BASE);
        assert_eq!(ba::ordenar_etapas(&[]), base);
        let elaboracao = v(&["Elaboração do DFD", "Elaboração do ETP", "Etapa Final"]);
        let mut abertura_lida: Vec<String> = base
            .iter()
            .map(|n| {
                if n == "Aprovação do COMPAC" {
                    n.to_uppercase()
                } else {
                    n.clone()
                }
            })
            .collect();
        abertura_lida[0] = "Validacao de  requisicao".into(); // grafia diferente, mesma etapa
        let etapas = ba::ordenar_etapas(&[elaboracao, abertura_lida]);
        let mut esperado = vec!["Validacao de  requisicao".to_string()];
        esperado.extend(base[1..5].iter().cloned());
        esperado.extend(v(&[
            "APROVAÇÃO DO COMPAC",
            "Etapa Final",
            "Elaboração do DFD",
            "Elaboração do ETP",
        ]));
        assert_eq!(etapas, esperado);
    }

    #[test]
    fn etapas_conhecidas_do_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let arq = tmp.path().join("1doc_cache.json");
        assert_eq!(ba::etapas_conhecidas(&arq), v(&ba::ETAPAS_BASE)); // sem cache
        std::fs::write(&arq, "{ quebrado").unwrap();
        assert_eq!(ba::etapas_conhecidas(&arq), v(&ba::ETAPAS_BASE));
        let elab = serde_json::json!(["Elaboração do DFD", "Elaboração do ETP", "Etapa Final"]);
        let mut det = serde_json::json!({"3|1/2026": {"etapas_quadro": elab}, "3|2/2026": {"etapas_quadro": elab},
                   "3|3/2026": {"etapas_quadro": []}, "3|4/2026": {"numero": "sem quadro"}});
        std::fs::write(
            &arq,
            serde_json::json!({"versao": "1.1", "det": det}).to_string(),
        )
        .unwrap();
        let e = ba::etapas_conhecidas(&arq);
        assert_eq!(e[e.len() - 2..], ["Elaboração do DFD", "Elaboração do ETP"]);
        det["3|5/2026"] = serde_json::json!({"etapas_quadro": ["Análise Jurídica", "Etapa Final"]}); // uma busca nova
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(
            &arq,
            serde_json::json!({"versao": "1.1", "det": det}).to_string(),
        )
        .unwrap();
        assert_eq!(
            ba::etapas_conhecidas(&arq).last().unwrap(),
            "Análise Jurídica"
        );
    }

    #[test]
    fn juntar_etapa() {
        for (antes, escolhida, varios, esperado) in [
            ("", "Pesquisa de Mercado", true, "Pesquisa de Mercado"),
            (
                "Pesquisa",
                "Pesquisa de Mercado",
                true,
                "Pesquisa de Mercado",
            ), // substitui
            (
                "Pesquisa de Mercado;",
                "Consulta Orçamentária",
                true,
                "Pesquisa de Mercado; Consulta Orçamentária",
            ),
            (
                "Pesquisa de Mercado; ",
                "pesquisa de mercado",
                true,
                "Pesquisa de Mercado",
            ), // não repete
            (
                "Pesquisa de Mercado;",
                "Consulta Orçamentária",
                false,
                "Consulta Orçamentária",
            ), // campo de uma só
        ] {
            assert_eq!(ba::juntar_etapa(antes, escolhida, varios), esperado);
        }
    }

    #[test]
    fn campos_de_etapa_viram_opcoes_da_busca() {
        for campo in ba::CAMPOS_ETAPA {
            assert!(
                ba::TEXTOS.iter().any(|(c, _)| *c == campo)
                    || ba::TEXTOS_MULTIPLOS.iter().any(|(c, _)| *c == campo)
            );
        }
        let f = form(&["entrada"])
            .com("etapa", "Pesquisa de Mercado; Consulta Orçamentária")
            .com("antes_de", "Pesquisa de Mercado");
        let argv = ba::montar_argv(&f, None, agora(), false).unwrap();
        assert_eq!(
            argv[2..],
            [
                "--antes-de",
                "Pesquisa de Mercado",
                "--etapa",
                "Pesquisa de Mercado",
                "--etapa",
                "Consulta Orçamentária"
            ]
        );
    }

    #[test]
    fn montar_argv_novo_processo() {
        let argv = |f: &Formulario| ba::montar_argv(f, None, agora(), false);
        let a = argv(
            &form(&["entrada"])
                .com("novo_processo", "Sim")
                .com("novo_processo_de", "01/11/2026")
                .com("novo_processo_ate", "-1"),
        )
        .unwrap();
        assert_eq!(
            a[2..],
            [
                "--novo-processo-de",
                "01/11/2026",
                "--novo-processo-ate",
                "-1",
                "--novo-processo"
            ]
        );
        assert_eq!(ba::validar(&a), None);
        assert_eq!(
            argv(&form(&["entrada"]).com("novo_processo", "Não")).unwrap()[2..],
            ["--sem-novo-processo"]
        );
        assert_eq!(
            argv(&form(&["entrada"]).com("novo_processo_de", "hoje")).unwrap()[2..],
            ["--novo-processo-de", "hoje"]
        );
        let erro = argv(
            &form(&["entrada"])
                .com("novo_processo", "Não")
                .com("novo_processo_ate", "hoje"),
        )
        .unwrap_err();
        assert!(erro.0.contains("não combina"), "{erro}");
        // a mesma regra vale para quem digita as opções
        let e = ba::validar(&v(&["--sem-novo-processo", "--novo-processo"])).unwrap();
        assert!(e.contains("não combina"), "{e}");
        let e = ba::validar(&v(&["--novo-processo-ate", "31/02/2026"])).unwrap();
        assert!(e.contains("data inválida"), "{e}");
    }

    #[test]
    fn novo_processo_so_conta_processo() {
        use ferramentas1doc::busca::registro::novos_processos;
        let det = serde_json::json!({
            "emissoes": [
                {"numero": "2/2026", "tipo": "Memorando", "data": "2026-11-05T10:00:00"},
                {"numero": "9/2026", "tipo": "Proc. Administrativo", "data": "2026-11-06T09:00:00"},
                {"numero": "8/2026", "tipo": "Proc. Administrativo", "data": "2026-11-01T09:00:00"}
            ],
            "derivados": ["2/2026", "9/2026", "8/2026"]
        });
        let nums: Vec<String> = novos_processos(Some(&det), None, true)
            .into_iter()
            .map(|n| n.numero)
            .collect();
        assert_eq!(nums, ["8/2026", "9/2026"]); // do mais antigo
        // detalhe gravado antes desta leitura: só os números, sem data
        let velho = novos_processos(
            Some(&serde_json::json!({"derivados": ["2/2026"]})),
            None,
            true,
        );
        assert_eq!(velho.len(), 1);
        assert_eq!((velho[0].numero.as_str(), velho[0].data), ("2/2026", None));
        assert!(novos_processos(Some(&serde_json::json!({})), None, true).is_empty());
    }

    #[test]
    fn citados_no_texto_e_nos_despachos() {
        use ferramentas1doc::busca::parse::citados;
        let fonte = r#"<div class="emissao_conteudo"><div class="texto_original">Referente à Fase Preparatória
      nº 30.500/2026, conforme a Lei nº 14.133/2021 e a requisição 77/2026. Este é o processo 41.000/2026.</div></div>
      <table class="despacho"><tr><td><div class="texto_original fr-view">A demanda prossegue no
      <a href="./?pg=doc/ver&hash=65edaaaa&origem=emissao_anexada">Proc. Administrativo 39.144/2026</a>;
      veja o <a href="./?pg=doc/ver&hash=7777bbbb">Memorando 512/2026</a> e o
      <a href="./?pg=doc/ver&hash=8888cccc">link</a>. Processo 39.144/2026 de novo.</div></td></tr></table>
      <div class="outro">processo 99.999/2026 fora dos blocos de texto</div>"#;
        let doc = ferramentas1doc::html::doc(fonte);
        assert_eq!(
            citados(&doc, "41.000/2026", "HASHPROPRIO"),
            [
                serde_json::json!({"numero": "30.500/2026", "hash": ""}),
                // link e número do mesmo processo: um registro
                serde_json::json!({"numero": "39.144/2026", "hash": "65EDAAAA"}),
                // link sem número: confere-se ao abrir
                serde_json::json!({"numero": "", "hash": "8888CCCC"}),
            ]
        );
    }

    const FASE: &str = "Fase Preparatória de Procedimento Licitatório";
    const ABERTURA: &str = "Abertura de Procedimento Licitatório";
    const ELAB: &str = "Elaboração de Documentação Preparatória";

    fn det(
        numero: &str,
        assunto: &str,
        emissao: &str,
        pai: &str,
        citados: &[&str],
    ) -> serde_json::Value {
        serde_json::json!({
            "numero": numero, "tipo_doc": "Proc. Administrativo", "quadro": assunto, "tipo": assunto,
            "pai": pai, "emissao": emissao, "hash": "", "etapa": "Validação de Requisição",
            "url": format!("u/{numero}"),
            "citados": citados.iter().map(|c| serde_json::json!({"numero": c, "hash": ""})).collect::<Vec<_>>(),
        })
    }

    #[test]
    fn aceita_mencao() {
        use ferramentas1doc::busca::registro::aceita_mencao;
        let fase = det("30.500/2026", FASE, "2026-11-01T09:00:00", "", &[]);
        let abertura = det("41.000/2026", ABERTURA, "2026-11-05T09:00:00", "", &[]);
        assert!(aceita_mencao(&fase, &abertura));
        assert!(aceita_mencao(
            &fase,
            &det(
                "41.000/2026",
                ABERTURA,
                "2026-11-05T09:00:00",
                "30.500/2026",
                &[]
            )
        ));
        assert!(aceita_mencao(
            &det("1/2026", ELAB, "2026-10-01T09:00:00", "", &[]),
            &fase
        ));
        assert!(aceita_mencao(
            &det("1/2026", "Outro assunto", "2026-10-01T09:00:00", "", &[]),
            &fase
        ));
        let mut memorando = abertura.clone();
        memorando["tipo_doc"] = serde_json::json!("Memorando");
        let rejeitados = [
            // emitida de outro processo
            det(
                "41.000/2026",
                ABERTURA,
                "2026-11-05T09:00:00",
                "28.000/2026",
                &[],
            ),
            // anterior à fase
            det("41.000/2026", ABERTURA, "2026-10-05T09:00:00", "", &[]),
            // fora da licitação
            det(
                "41.000/2026",
                "Pagamento de Fornecedor",
                "2026-11-05T09:00:00",
                "",
                &[],
            ),
            // outra fase: referência
            det("30.900/2026", FASE, "2026-11-05T09:00:00", "", &[]),
            memorando,
            // ele mesmo
            det("30.500/2026", ABERTURA, "2026-11-05T09:00:00", "", &[]),
        ];
        for d in &rejeitados {
            assert!(!aceita_mencao(&fase, d), "{d}");
        }
    }

    #[test]
    fn novos_processos_por_vinculo() {
        use ferramentas1doc::busca::registro::{Universo, novos_processos, texto_novos};
        let mut fase = det(
            "30.500/2026",
            FASE,
            "2026-11-01T09:00:00",
            "",
            &["41.000/2026", "40.001/2026"],
        );
        fase["emissoes"] = serde_json::json!([
            {"numero": "40.001/2026", "tipo": "Proc. Administrativo", "data": "2026-11-02T10:00:00"}
        ]);
        let outros = [
            fase.clone(),
            det(
                "40.001/2026",
                ABERTURA,
                "2026-11-02T10:00:00",
                "30.500/2026",
                &["30.500/2026"],
            ),
            det(
                "41.000/2026",
                ABERTURA,
                "2026-11-05T09:00:00",
                "",
                &["30.500/2026"],
            ),
            det(
                "41.050/2026",
                ABERTURA,
                "2026-11-06T09:00:00",
                "30.500/2026",
                &[],
            ),
            det(
                "41.100/2026",
                ABERTURA,
                "2026-11-07T09:00:00",
                "28.000/2026",
                &["30.500/2026"],
            ),
        ];
        let uni = Universo::novo(outros.iter());
        let novos = novos_processos(Some(&fase), Some(&uni), true);
        // emissão normal basta (a menção e a origem do mesmo processo não se somam); menções se somam entre si
        let v: Vec<(&str, Vec<&str>)> = novos
            .iter()
            .map(|n| (n.numero.as_str(), n.vinculos.clone()))
            .collect();
        assert_eq!(
            v,
            [
                ("40.001/2026", vec!["emissão"]),
                ("41.000/2026", vec!["citado neste", "cita este"]),
                ("41.050/2026", vec!["origem"]),
            ]
        );
        assert_eq!(
            novos[1].data,
            NaiveDate::from_ymd_opt(2026, 11, 5)
                .unwrap()
                .and_hms_opt(9, 0, 0)
        );
        assert_eq!(novos[1].etapa, "Validação de Requisição");
        assert_eq!(
            texto_novos(&novos),
            "40.001/2026 (emissão), 41.000/2026 (citado neste + cita este), 41.050/2026 (origem)"
        );
        // só a emissão normal: fica a origem, saem as menções
        let so_normal: Vec<String> = novos_processos(Some(&fase), Some(&uni), false)
            .into_iter()
            .map(|n| n.numero)
            .collect();
        assert_eq!(so_normal, ["40.001/2026", "41.050/2026"]);
    }

    #[test]
    fn montar_argv_erros() {
        assert!(
            ba::montar_argv(&form(&[]), None, agora(), false)
                .unwrap_err()
                .0
                .contains("caixa")
        );
        assert!(
            ba::montar_argv(
                &form(&["entrada"]).com("dias_etapa_min", "dez"),
                None,
                agora(),
                false
            )
            .unwrap_err()
            .0
            .contains("inteiro")
        );
        assert!(
            ba::validar(&v(&["--emissao-de", "31/31/2026"]))
                .unwrap()
                .contains("data inválida")
        );
        assert!(
            ba::validar(&v(&["--nao-existe"]))
                .unwrap()
                .contains("--nao-existe")
        );
    }

    #[test]
    fn abreviacoes_e_negativos_como_no_argparse() {
        // abreviação ambígua é recusada; a única vale, como no argparse
        assert!(
            ba::validar(&v(&["--sem-mov-m", "5"]))
                .unwrap()
                .contains("--sem-mov-m")
        );
        assert_eq!(
            ba::validar(&v(&[
                "--etapa-ex",
                "--emissao-de",
                "-30",
                "--dias-etapa-min",
                "3"
            ])),
            None
        );
        assert!(
            ba::validar(&v(&["--fase", "xyz"]))
                .unwrap()
                .contains("fase: Elaboração ou Abertura")
        );
        assert!(
            ba::validar(&v(&["--dias-etapa-min", "dez"]))
                .unwrap()
                .contains("--dias-etapa-min")
        );
    }
}

mod faltantes_funcoes {
    use super::{d, hoje};
    use ferramentas1doc::faltantes;
    use ferramentas1doc::verificadores::Item;

    fn itens(anos: &[&str]) -> Vec<Item> {
        anos.iter()
            .map(|a| Item {
                ano: a.to_string(),
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn resolver_periodo() {
        let it = itens(&["2026", "2024"]);
        assert_eq!(
            faltantes::resolver_periodo(None, None, &it, hoje()).unwrap(),
            (d(2024, 1, 1), hoje())
        );
        assert_eq!(
            faltantes::resolver_periodo(None, None, &[], hoje()).unwrap(),
            (d(2026, 1, 1), hoje())
        );
        // fim no futuro: hoje
        assert_eq!(
            faltantes::resolver_periodo(Some(d(2025, 6, 1)), Some(d(2027, 1, 1)), &it, hoje())
                .unwrap(),
            (d(2025, 6, 1), hoje())
        );
        let e = faltantes::resolver_periodo(None, Some(d(2023, 12, 31)), &it, hoje()).unwrap_err();
        assert!(e.to_string().contains("Período vazio"));
    }

    #[test]
    fn data_de_emissao_na_listagem() {
        let html = concat!(
            r#"<table><tr><td><a class="link_emissao_a" href="./?pg=doc/ver&hash=AAAA1111BBBB2222CCCC">"#,
            r#"Proc. Administrativo 31.632/2026 <small class="data">05/11/2026 09:15</small></a></td></tr>"#,
            r#"<tr><td><a class="link_emissao_a" href="./?pg=doc/ver&hash=DDDD3333EEEE4444FFFF">"#,
            r#"Proc. Administrativo 31.700/2026</a></td></tr></table>"#
        );
        let e: Vec<_> = faltantes::linhas_listagem(html)
            .into_iter()
            .map(|l| l.emissao)
            .collect();
        assert_eq!(e, [Some(d(2026, 11, 5)), None]);
    }

    #[test]
    fn cancelamento() {
        assert_eq!(
            faltantes::posicao_cancelamento("Solicito o cancelamento do processo"),
            Some(11)
        );
        assert!(!faltantes::menciona_cancelamento(
            "Não há cancelamento previsto"
        ));
        assert!(!faltantes::menciona_cancelamento("Segue sem cancelamento."));
        assert!(faltantes::menciona_cancelamento(
            "Processo CANCELADO por desistência"
        ));
        let t = "a ".repeat(200) + "cancelar " + &"b ".repeat(200);
        let tr = faltantes::trecho_em_volta(&t, faltantes::posicao_cancelamento(&t).unwrap(), 150);
        assert!(tr.starts_with("… ") && tr.ends_with(" …") && tr.contains("cancelar"));
    }
}

/// O despacho pode vir com outras classes junto de texto_original.
#[test]
fn citacoes_le_despacho_com_varias_classes() {
    let html = concat!(
        r#"<table class="relative despacho"><tr><td class="celula_despacho">"#,
        r#"<div class="texto_original">Segue para o processo nº 31.700/2026.</div></td></tr></table>"#,
        r#"<table class="relative despacho"><tr><td class="celula_despacho">"#,
        r#"<div class="texto_original fr-view">Emitido o Proc. Administrativo 31.800/2026 "#,
        r#"(<a href="./?pg=doc/ver&hash=AAAA1111BBBB2222CCCC">ver</a>).</div></td></tr></table>"#,
        r#"<table class="relative despacho"><tr><td class="celula_despacho">"#,
        r#"<div class="fr-view texto_original">Abertura: processo 31.900/2026.</div></td></tr></table>"#,
        r#"<p class="rodape">Processo 32.000/2026 fora dos despachos</p>"#,
    );
    let (hashes, numeros) = ferramentas1doc::verificadores::citacoes(html, "31.632/2026", "2026");
    assert_eq!(hashes, ["AAAA1111BBBB2222CCCC"]);
    assert_eq!(numeros, ["31.700/2026", "31.800/2026", "31.900/2026"]);
}

/// activeTab apontando para uma aba que não existe (arquivo editado por
/// outro programa): fica a primeira, sem pânico.
#[test]
fn aba_ativa_fora_do_intervalo_fica_a_primeira() {
    use ferramentas1doc::verificadores::aba_ativa;
    let mut wb = umya_spreadsheet::new_file();
    wb.new_sheet("Acompanhamento").unwrap();
    wb.workbook_view_mut().set_active_tab(1);
    assert_eq!(aba_ativa(&wb).as_deref(), Some("Acompanhamento"));
    wb.workbook_view_mut().set_active_tab(9);
    assert_eq!(aba_ativa(&wb).as_deref(), Some("Sheet1"));
}
