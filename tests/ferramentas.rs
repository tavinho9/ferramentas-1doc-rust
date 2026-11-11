//! As quatro ferramentas de ponta a ponta contra o 1Doc de mentira
//! (porta de tests/test_ferramentas.py).

mod comum;

use chrono::Duration;
use ferramentas1doc::processos_do_dia as pdd;
use ferramentas1doc::{Erro, sessao};

fn opcoes() -> pdd::Opcoes {
    pdd::Opcoes {
        pausa: 0.0,
        ..Default::default()
    }
}

#[test]
fn processos_do_dia() {
    let fake = comum::Fake1Doc::novo();
    let d = comum::dados(&fake.base);
    let hoje = comum::hoje();
    let caminho = pdd::executar(&d.amb, hoje, &comum::cred(), opcoes())
        .unwrap()
        .unwrap();
    assert_eq!(
        caminho,
        d.pasta_saida()
            .join(format!("processos novos {}.xlsx", hoje.format("%d.%m")))
    );
    let linhas = comum::valores(&caminho, None);
    assert_eq!(
        linhas[0],
        ["Nº Processo", "Prioridade", "Objeto", "Etapa", "Secretaria"]
    );
    // hoje: só entra quem está na etapa atual há menos de 24h (P1, "Esta aqui 2 horas")
    assert_eq!(
        linhas[1..]
            .iter()
            .map(|l| l[0].as_str())
            .collect::<Vec<_>>(),
        ["31.632/2026"]
    );
    assert_eq!(
        linhas[1][2],
        "Aquisição de medicamentos para a rede municipal de saúde"
    ); // campo da abertura
    assert_eq!(linhas[1][3], "Pesquisa de Mercado — Abertura");
    assert_eq!(linhas[1][4], "Secretaria de Saúde");
    assert_eq!(
        comum::link(&caminho, None, 1, 2).as_deref(),
        Some(format!("{}/?pg=doc/ver&hash=AAAA1111BBBB2222CCCC&itd=3", fake.base).as_str())
    );
    let js: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(d.pasta_saida().join("processos_do_dia.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(js["total"], 2);
    assert_eq!(js["dia"], hoje.format("%d/%m/%Y").to_string());
    let p2 = js["itens"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["numero"] == "31.700/2026")
        .unwrap();
    assert_eq!(p2["incluir"], false);
    assert_eq!(p2["motivo"], "etapa atual há 1 dia ou mais");
    assert!(fake.chaves().contains(&("GET".into(), "caixa".into())));
}

#[test]
fn processos_do_dia_rejeita_futuro() {
    let fake = comum::Fake1Doc::novo();
    let d = comum::dados(&fake.base);
    let e = pdd::executar(
        &d.amb,
        comum::hoje() + Duration::days(1),
        &comum::cred(),
        opcoes(),
    )
    .unwrap_err();
    assert!(
        matches!(&e, Erro::Encerrar(m) if m.contains("ainda não chegou")),
        "{e:?}"
    );
}

#[test]
fn processos_do_dia_sessao_caida_apaga_a_salva() {
    let fake = comum::Fake1Doc::novo();
    fake.deslogar();
    let d = comum::dados(&fake.base);
    sessao::salvar(&d.amb, &comum::cred()).unwrap();
    assert_eq!(
        pdd::executar(&d.amb, comum::hoje(), &comum::cred(), opcoes()).unwrap(),
        None
    );
    assert!(sessao::carregar_salva(&d.amb).is_none());
}

#[test]
fn cancelar_interrompe() {
    let fake = comum::Fake1Doc::novo();
    let d = comum::dados(&fake.base);
    d.amb.cancelar.pedir();
    assert_eq!(
        pdd::executar(&d.amb, comum::hoje(), &comum::cred(), opcoes()).unwrap_err(),
        Erro::Cancelado
    );
}

#[test]
fn busca() {
    use ferramentas1doc::{busca, busca_argv};
    let fake = comum::Fake1Doc::novo();
    let d = comum::dados(&fake.base);
    let form = busca_argv::Formulario {
        caixas: vec!["entrada".into()],
        ..Default::default()
    }
    .com("etapa", "Pesquisa de Mercado")
    .marcar("csv");
    let mut argv = busca_argv::montar_argv(
        &form,
        Some(&d.pasta_saida()),
        chrono::Local::now().naive_local(),
        false,
    )
    .unwrap();
    argv.extend(["--pausa".into(), "0".into()]);
    let res = busca::main(&d.amb, &argv, Some(&comum::cred()))
        .unwrap()
        .unwrap();
    assert_eq!(
        res.regs
            .iter()
            .map(|r| r.numero.as_str())
            .collect::<Vec<_>>(),
        ["31.632/2026"]
    );
    let arquivo = res.arquivo.unwrap();
    assert_eq!(arquivo.parent().unwrap(), d.pasta_saida());
    let col_a: Vec<String> = comum::valores(&arquivo, Some("Resultado"))
        .into_iter()
        .map(|l| l[0].clone())
        .collect();
    assert_eq!(col_a, ["Nº Processo", "31.632/2026"]);
    assert!(
        std::fs::read_dir(d.pasta_saida()).unwrap().any(|e| e
            .unwrap()
            .path()
            .extension()
            .is_some_and(|x| x == "csv"))
    );
    assert!(d.pasta_dados().join("1doc_cache.json").is_file()); // cache na pasta de dados

    // a mesma consulta sem acessar o 1Doc, só com o cache
    fake.deslogar();
    let argv: Vec<String> = [
        "--offline",
        "--antes-de",
        "Pesquisa de Mercado",
        "--sem-xlsx",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let res2 = busca::main(&d.amb, &argv, None).unwrap().unwrap();
    assert_eq!(
        res2.regs
            .iter()
            .map(|r| r.numero.as_str())
            .collect::<Vec<_>>(),
        ["31.700/2026"]
    );
}

#[test]
fn busca_novo_processo() {
    use ferramentas1doc::{busca, busca_argv};
    let tres_dias = (comum::hoje() - Duration::days(3))
        .and_hms_opt(14, 30, 0)
        .unwrap();
    let hoje_8h = comum::hoje().and_hms_opt(8, 0, 0).unwrap();
    let p1: &'static comum::Processo = &comum::PROCESSOS[0];
    let eventos =
        comum::evento_emissao(
            "Proc. Administrativo",
            "40.001/2026",
            "9999AAAA0000BBBB1111",
            tres_dias,
        ) + &comum::evento_emissao("Memorando", "512/2026", "8888CCCC9999DDDD0000", hoje_8h);
    // P1 com dois documentos emitidos a partir dele: um processo e um memorando
    let fake = comum::Fake1Doc::com(Some(Box::new(move |p| {
        (p.query.get("pg").map(String::as_str) == Some("doc/ver")
            && p.query
                .get("hash")
                .is_some_and(|h| h.eq_ignore_ascii_case(p1.hash)))
        .then(|| comum::Resposta::html(comum::detalhe_com(p1, &eventos)))
    })));
    let d = comum::dados(&fake.base);
    let numeros =
        |r: &busca::Resultado| -> Vec<String> { r.regs.iter().map(|x| x.numero.clone()).collect() };
    let args = |v: &[&str]| -> Vec<String> { v.iter().map(|s| s.to_string()).collect() };
    // número de série do Excel (dias desde 30/12/1899)
    let serie = |t: chrono::NaiveDateTime| {
        let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        (t - base).num_seconds() as f64 / 86400.0
    };
    let mesma_data = |celula: &str, t| (celula.parse::<f64>().unwrap() - serie(t)).abs() < 1e-6;

    let form = busca_argv::Formulario {
        caixas: vec!["entrada".into()],
        ..Default::default()
    }
    .com("novo_processo", "Sim");
    let mut argv = busca_argv::montar_argv(
        &form,
        Some(&d.pasta_saida()),
        chrono::Local::now().naive_local(),
        false,
    )
    .unwrap();
    argv.extend(args(&["--pausa", "0"]));
    let res = busca::main(&d.amb, &argv, Some(&comum::cred()))
        .unwrap()
        .unwrap();
    assert_eq!(numeros(&res), ["31.632/2026"]);
    assert_eq!(res.regs[0].novo_processo_em, Some(tres_dias));
    assert_eq!(res.regs[0].derivados, "40.001/2026, 512/2026"); // a coluna de antes segue com tudo
    let arquivo = res.arquivo.clone().unwrap();
    let linhas = comum::valores(&arquivo, Some("Resultado"));
    let col = linhas[0]
        .iter()
        .position(|c| c == "Novo processo emitido em")
        .unwrap();
    assert!(mesma_data(&linhas[1][col], tres_dias), "{}", linhas[1][col]);
    // só o processo: o memorando emitido a partir dele fica de fora
    let novos = comum::valores(&arquivo, Some("Novos processos"));
    assert_eq!(
        novos[0],
        [
            "Nº Processo",
            "Objeto",
            "Novo processo",
            "Emitido em",
            "Autor",
            "Setor",
            "Etapa atual do processo"
        ]
    );
    assert_eq!(novos.len(), 2);
    assert_eq!(
        [&novos[1][..3], &novos[1][4..]].concat(),
        [
            "31.632/2026",
            "Aquisição de medicamentos",
            "40.001/2026",
            "Maria da Silva",
            "SS-DAF",
            "Pesquisa de Mercado"
        ]
    );
    assert!(mesma_data(&novos[1][3], tres_dias), "{}", novos[1][3]);
    let link = comum::link(&arquivo, Some("Novos processos"), 3, 2).unwrap();
    assert!(link.contains("hash=9999AAAA0000BBBB1111"), "{link}");
    let filtros = &res.params.iter().find(|p| p.0 == "Filtros").unwrap().1;
    assert!(
        filtros.contains("emitiu novo processo a partir dele"),
        "{filtros}"
    );

    // sem acessar o 1Doc
    let antes = fake.chaves().len();
    let offline = |v: &[&str]| {
        let mut a = args(&["--offline", "--sem-xlsx"]);
        a.extend(args(v));
        busca::main(&d.amb, &a, None).unwrap().unwrap()
    };
    assert_eq!(
        numeros(&offline(&[
            "--novo-processo-de",
            "-5",
            "--novo-processo-ate",
            "ontem"
        ])),
        ["31.632/2026"]
    );
    // o de hoje é memorando, não processo
    assert!(offline(&["--novo-processo-de", "hoje"]).regs.is_empty());
    assert_eq!(numeros(&offline(&["--sem-novo-processo"])), ["31.700/2026"]);
    // com período, a aba Novos processos diz quais caíram nele
    let saida = d.pasta_saida().join("periodo.xlsx");
    let saida_txt = saida.to_string_lossy().into_owned();
    busca::main(
        &d.amb,
        &args(&[
            "--offline",
            "--saida",
            &saida_txt,
            "--novo-processo-de",
            "-5",
        ]),
        None,
    )
    .unwrap();
    let col_e: Vec<String> = comum::valores(&saida, Some("Novos processos"))
        .into_iter()
        .map(|l| l[4].clone())
        .collect();
    assert_eq!(col_e, ["No período filtrado?", "sim"]);
    assert_eq!(fake.chaves().len(), antes);

    // Cache gravado antes de a busca ler as emissões (só os números em "derivados")
    let arq = d.pasta_dados().join("1doc_cache.json");
    let mut cache: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&arq).unwrap()).unwrap();
    for det in cache["det"].as_object_mut().unwrap().values_mut() {
        det.as_object_mut().unwrap().remove("emissoes").unwrap();
    }
    std::fs::write(&arq, cache.to_string()).unwrap();
    // sem acessar o 1Doc, o período não tem como ser conferido: lacuna declarada, não chute
    let res = offline(&["--novo-processo-de", "-5"]);
    assert!(res.regs.is_empty());
    let falhas: Vec<(&str, &str)> = res
        .falhas
        .iter()
        .map(|f| (f.numero.as_str(), f.motivo.as_str()))
        .collect();
    assert_eq!(
        falhas,
        [(
            "31.632/2026",
            "cache sem a data do novo processo (modo offline)"
        )]
    );
    assert_eq!(numeros(&offline(&["--novo-processo"])), ["31.632/2026"]);
    // com acesso, só o processo que tem derivado é reaberto; o outro vem do cache
    let antes = fake.chaves().len();
    let res = busca::main(
        &d.amb,
        &args(&["--novo-processo-de", "-5", "--sem-xlsx", "--pausa", "0"]),
        Some(&comum::cred()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(numeros(&res), ["31.632/2026"]);
    assert!(res.falhas.is_empty());
    let abertos = fake.chaves()[antes..]
        .iter()
        .filter(|c| c.1 == "doc/ver")
        .count();
    assert_eq!(abertos, 1);
}

mod faltantes_testes {
    use super::comum;
    use chrono::Duration;
    use ferramentas1doc::{Erro, faltantes};
    use std::collections::HashSet;

    fn op() -> faltantes::Opcoes {
        faltantes::Opcoes {
            pausa: 0.0,
            sem_navegador: true,
            ..Default::default()
        }
    }

    fn br(d: chrono::NaiveDate) -> String {
        d.format("%d/%m/%Y").to_string()
    }

    fn intervalos(fake: &comum::Fake1Doc) -> HashSet<String> {
        fake.intervalos().into_iter().flatten().collect()
    }

    #[test]
    fn faltantes() {
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(&d.pasta_dados().join("acomp.xlsx"), &["31.632/2026"]);
        let destino = faltantes::executar(&d.amb, &pl, None, &comum::cred(), op())
            .unwrap()
            .unwrap();
        assert_eq!(destino.parent().unwrap(), d.pasta_saida());
        let linhas = comum::valores(&destino, Some("Faltantes"));
        // P2 está antes da Pesquisa de Mercado e fora da planilha; P1 está na planilha
        assert_eq!(
            linhas[1..]
                .iter()
                .map(|l| l[0].as_str())
                .collect::<Vec<_>>(),
            ["31.700/2026"]
        );
        assert_eq!(linhas[1][2], "Reforma da Escola Municipal Exemplo");
        assert!(d.pasta_dados().join("hash_cache_1doc.json").is_file());
        // período padrão: de 1º/jan do ano mais antigo da planilha até hoje
        assert_eq!(
            intervalos(&fake),
            HashSet::from([format!("01/01/2026 - {}", br(comum::hoje()))])
        );
    }

    #[test]
    fn faltantes_periodo_escolhido() {
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(&d.pasta_dados().join("acomp.xlsx"), &["31.632/2026"]);
        let hoje = comum::hoje();
        let o = faltantes::Opcoes {
            de: Some(hoje - Duration::days(10)),
            ate: Some(hoje + Duration::days(5)),
            ..op()
        }; // fim no futuro vira hoje
        let destino = faltantes::executar(&d.amb, &pl, None, &comum::cred(), o)
            .unwrap()
            .unwrap();
        let linhas = comum::valores(&destino, Some("Faltantes"));
        assert_eq!(
            linhas[1..]
                .iter()
                .map(|l| l[0].as_str())
                .collect::<Vec<_>>(),
            ["31.700/2026"]
        );
        assert_eq!(
            intervalos(&fake),
            HashSet::from([format!("{} - {}", br(hoje - Duration::days(10)), br(hoje))])
        );
    }

    #[test]
    fn faltantes_periodo_sem_processos() {
        // Os dois processos foram emitidos hoje. Com o período terminando
        // ontem nenhum entra, mesmo com o servidor ignorando o intervalo.
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(&d.pasta_dados().join("acomp.xlsx"), &["31.632/2026"]);
        let ontem = comum::hoje() - Duration::days(1);
        let o = faltantes::Opcoes {
            de: Some(ontem - Duration::days(30)),
            ate: Some(ontem),
            ..op()
        };
        assert_eq!(
            faltantes::executar(&d.amb, &pl, None, &comum::cred(), o).unwrap(),
            None
        );
        assert_eq!(
            intervalos(&fake),
            HashSet::from([format!(
                "{} - {}",
                br(ontem - Duration::days(30)),
                br(ontem)
            )])
        );
        assert!(!fake.chaves().contains(&("GET".into(), "doc/ver".into()))); // nenhum processo aberto
        assert!(
            d.saida_texto()
                .contains("Nenhum Proc. Administrativo emitido de")
        );
        assert_eq!(std::fs::read_dir(d.pasta_saida()).unwrap().count(), 0);
    }

    #[test]
    fn faltantes_todos_ja_na_planilha() {
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(
            &d.pasta_dados().join("acomp.xlsx"),
            &["31.632/2026", "31.700/2026"],
        );
        assert_eq!(
            faltantes::executar(&d.amb, &pl, None, &comum::cred(), op()).unwrap(),
            None
        );
        assert!(d.saida_texto().contains("já estão na planilha"));
    }

    #[test]
    fn faltantes_periodo_vazio() {
        // Só a data final, antes do início padrão (1º/jan de 2026, pela planilha).
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(&d.pasta_dados().join("acomp.xlsx"), &["31.632/2026"]);
        let o = faltantes::Opcoes {
            ate: chrono::NaiveDate::from_ymd_opt(2025, 12, 31),
            ..op()
        };
        let e = faltantes::executar(&d.amb, &pl, None, &comum::cred(), o).unwrap_err();
        assert!(
            matches!(&e, Erro::Encerrar(m) if m.contains("Período vazio")),
            "{e:?}"
        );
    }

    #[test]
    fn planilha_inexistente() {
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let e = faltantes::executar(
            &d.amb,
            &d.pasta_dados().join("nao_existe.xlsx"),
            None,
            &comum::cred(),
            op(),
        )
        .unwrap_err();
        assert!(
            matches!(&e, Erro::Encerrar(m) if m.contains("não encontrado")),
            "{e:?}"
        );
    }
}

mod verificar_testes {
    use super::comum;
    use ferramentas1doc::verificar_planilha as vp;

    fn op() -> vp::Opcoes {
        vp::Opcoes {
            pausa: 0.0,
            sem_navegador: true,
            ..Default::default()
        }
    }

    #[test]
    fn verificar_planilha() {
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(
            &d.pasta_dados().join("acomp.xlsx"),
            &["31.632/2026", "31.700/2026"],
        );
        let destino = vp::executar(&d.amb, &pl, Some("Acompanhamento"), &comum::cred(), op())
            .unwrap()
            .unwrap();
        let linhas = comum::valores(&destino, None);
        // só P1 chegou à Pesquisa de Mercado; colunas novas no fim
        assert_eq!(
            linhas[1..]
                .iter()
                .map(|l| l[0].as_str())
                .collect::<Vec<_>>(),
            ["31.632/2026"]
        );
        assert_eq!(
            linhas[0][5..],
            [
                "Etapa atual (1Doc)",
                "Processo pai",
                "Processo filho",
                "Objeto (abertura)",
                "Observação"
            ]
        );
        assert_eq!(linhas[1][5], "Pesquisa de Mercado");
        assert_eq!(
            linhas[1][8],
            "Aquisição de medicamentos para a rede municipal de saúde"
        );
        assert_eq!(
            comum::link(&destino, None, 1, 2).as_deref(),
            Some(format!("{}/?pg=doc/ver&hash=AAAA1111BBBB2222CCCC&itd=3", fake.base).as_str())
        );
        let cache: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(d.pasta_dados().join("hash_cache_1doc.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(cache["31.632/2026"], "AAAA1111BBBB2222CCCC"); // localizado por HTTP e guardado
    }

    #[test]
    fn aba_inexistente() {
        let fake = comum::Fake1Doc::novo();
        let d = comum::dados(&fake.base);
        let pl = comum::planilha(&d.pasta_dados().join("acomp.xlsx"), &["31.632/2026"]);
        let e = vp::executar(&d.amb, &pl, Some("Outra"), &comum::cred(), op()).unwrap_err();
        assert!(e.to_string().contains("Aba 'Outra' não existe"), "{e}");
    }
}
