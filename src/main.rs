// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

pub(crate) mod graph;
pub(crate) mod a_starr;
pub(crate) mod structs;
pub(crate) mod gbfs;
pub(crate) mod mapa;

use std::io::{self, Write};
use std::time::Duration;

use graph::Grafo;

/// OSMID padrão de origem para rota de demonstração rápida.
const DEMO_ORIGEM: u64 = 259576101;
/// OSMID padrão de destino para rota de demonstração rápida.
const DEMO_DESTINO: u64 = 259577028;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // 1. Carrega ou desserializa a malha viária de Ribeirão Preto
    println!("Carregando Grafo de Ribeirão Preto...");
    let inicio_carga = std::time::Instant::now();
    let ribeirao_graph = Grafo::new()?;
    let tempo_carga = inicio_carga.elapsed();
    println!(
        "Grafo carregado em {} ({} nós, {} arestas).\n",
        formatar_tempo(tempo_carga),
        ribeirao_graph.total_nos(),
        ribeirao_graph.arestas_destino.len()
    );

    // 2. Modo direto por linha de comando:
    //    cargo run -- <origem> <destino>
    //    cargo run -- <origem> <destino> <peso_g> <peso_h>
    if args.len() == 3 {
        match (args[1].parse::<u64>(), args[2].parse::<u64>()) {
            (Ok(origem), Ok(destino)) => {
                executar_e_exibir_buscas(&ribeirao_graph, origem, destino, 0.5, 0.5)?;
                perguntar_e_abrir_mapa("mapa_rotas.html")?;
                return Ok(());
            }
            _ => {
                eprintln!("Uso: {} <osmid_origem> <osmid_destino> [<peso_g> <peso_h>]", args[0]);
                std::process::exit(1);
            }
        }
    } else if args.len() == 5 {
        match (
            args[1].parse::<u64>(),
            args[2].parse::<u64>(),
            args[3].replace(',', ".").parse::<f64>(),
            args[4].replace(',', ".").parse::<f64>(),
        ) {
            (Ok(origem), Ok(destino), Ok(peso_g), Ok(peso_h)) => {
                executar_e_exibir_buscas(&ribeirao_graph, origem, destino, peso_g, peso_h)?;
                perguntar_e_abrir_mapa("mapa_rotas.html")?;
                return Ok(());
            }
            _ => {
                eprintln!("Uso: {} <osmid_origem> <osmid_destino> <peso_g> <peso_h>", args[0]);
                std::process::exit(1);
            }
        }
    }

    // 3. Menu interativo principal
    menu_interativo(&ribeirao_graph)?;

    Ok(())
}

/// Menu interativo de opções no terminal.
fn menu_interativo(grafo: &Grafo) -> io::Result<()> {
    loop {
        println!("==========================================");
        println!("SISTEMA DE ROTEAMENTO - RIBEIRÃO PRETO");
        println!("IA 2026 - Broken Ways");
        println!("==========================================");
        println!("[1] Pesquisar Rotas (informar Origem e Destino)");
        println!("[2] Rota de Demonstração ({} -> {})", DEMO_ORIGEM, DEMO_DESTINO);
        println!("[3] Abrir Último Mapa no Navegador");
        println!("[0] Sair");
        println!("==========================================");
        print!("Selecione uma opção: ");
        io::stdout().flush()?;

        let mut opcao = String::new();
        if io::stdin().read_line(&mut opcao).is_err() {
            break;
        }

        match opcao.trim() {
            "1" => {
                loop_consultas(grafo)?;
            }
            "2" => {
                println!("\nExecutando rota de demonstração padrão...");
                executar_e_exibir_buscas(grafo, DEMO_ORIGEM, DEMO_DESTINO, 0.5, 0.5)?;
                let abrir = perguntar_e_abrir_mapa("mapa_rotas.html")?;
                if !abrir {
                    println!("Iniciando nova consulta...");
                    loop_consultas(grafo)?;
                }
            }
            "3" => {
                let caminho_mapa = "mapa_rotas.html";
                if std::path::Path::new(caminho_mapa).exists() {
                    println!("\nAbrindo '{}' no navegador padrão...", caminho_mapa);
                    if let Err(e) = mapa::abrir_no_navegador(caminho_mapa) {
                        eprintln!("Não foi possível abrir o navegador: {}", e);
                    }
                } else {
                    println!("\nNenhum mapa foi gerado ainda. Realize uma busca primeiro!\n");
                }
            }
            "0" | "q" | "exit" => {
                println!("\nEncerrando o programa. Até logo!\n");
                break;
            }
            _ => {
                println!("\nOpção inválida. Digite 1, 2, 3 ou 0.\n");
            }
        }
    }

    Ok(())
}

/// Loop contínuo de consultas de rotas: a cada busca, pergunta se deseja abrir o mapa;
/// em caso de 'n', inicia imediatamente outra consulta.
fn loop_consultas(grafo: &Grafo) -> io::Result<()> {
    loop {
        println!("\n==========================================");
        println!("NOVA CONSULTA DE ROTA");
        println!("==========================================");

        let origem = match ler_osmid("Digite o OSMID de Origem (ou ENTER para voltar ao menu): ", grafo)? {
            Some(id) => id,
            None => break,
        };

        let destino = match ler_osmid("Digite o OSMID de Destino (ou ENTER para voltar ao menu): ", grafo)? {
            Some(id) => id,
            None => break,
        };

        // Etapa de configuração dos pesos de balanceamento do A*
        let (peso_g, peso_h) = ler_pesos_astar()?;

        println!();
        executar_e_exibir_buscas(grafo, origem, destino, peso_g, peso_h)?;

        let abriu = perguntar_e_abrir_mapa("mapa_rotas.html")?;
        if !abriu {
            println!("Mapa não aberto. Iniciando outra consulta...");
        }
    }

    Ok(())
}

/// Solicita e lê os pesos de balanceamento w_g e w_h para a função de avaliação do A*:
/// f(n) = w_g * g(n) + w_h * h(n).
/// Permite pressionar ENTER diretamente para aceitar os valores padrão (0.5 e 0.5).
fn ler_pesos_astar() -> io::Result<(f64, f64)> {
    println!("\n--- Configuração de Balanceamento do A* [f(n) = w_g · g(n) + w_h · h(n)] ---");
    println!("Pressione ENTER para aceitar o valor padrão indicado.");

    let peso_g = ler_peso("Digite o peso do custo acumulado g(n) [w_g] (padrão: 0.5): ", 0.5)?;
    let peso_h = ler_peso("Digite o peso da heurística h(n) [w_h] (padrão: 0.5): ", 0.5)?;

    println!("✔ Pesos configurados para o A*: w_g = {:.2}, w_h = {:.2}", peso_g, peso_h);
    Ok((peso_g, peso_h))
}

/// Lê e valida um peso em ponto flutuante não-negativo (f64 >= 0.0).
/// Se o usuário teclar ENTER vazio, adota o valor default `padrao`.
fn ler_peso(prompt: &str, padrao: f64) -> io::Result<f64> {
    loop {
        print!("{}", prompt);
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let texto = buffer.trim();

        if texto.is_empty() {
            return Ok(padrao);
        }

        let texto_normalizado = texto.replace(',', ".");
        match texto_normalizado.parse::<f64>() {
            Ok(val) if val >= 0.0 => return Ok(val),
            Ok(_) => {
                println!("❌ O peso não pode ser negativo. Digite um valor >= 0.0.");
            }
            Err(_) => {
                println!(
                    "❌ Formato inválido. Digite um número decimal (ex.: 0.5 ou 1.0) ou tecle ENTER para aceitar {}.",
                    padrao
                );
            }
        }
    }
}

/// Pergunta se o usuário deseja abrir o mapa no navegador.
/// Retorna `true` se abriu, `false` se o usuário recusou ('n').
fn perguntar_e_abrir_mapa(caminho_mapa: &str) -> io::Result<bool> {
    print!("Deseja abrir o mapa no navegador? [S/n]: ");
    io::stdout().flush()?;

    let mut resp = String::new();
    io::stdin().read_line(&mut resp)?;
    let r = resp.trim().to_lowercase();

    if r.is_empty() || r == "s" || r == "sim" || r == "y" || r == "yes" {
        println!("Abrindo mapa no navegador padrão...");
        if let Err(e) = mapa::abrir_no_navegador(caminho_mapa) {
            eprintln!("Não foi possível abrir o navegador: {}", e);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Lê e valida um OSMID digitado pelo usuário.
fn ler_osmid(prompt: &str, grafo: &Grafo) -> io::Result<Option<u64>> {
    loop {
        print!("{}", prompt);
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let texto = buffer.trim();

        if texto.is_empty() {
            return Ok(None);
        }

        match texto.parse::<u64>() {
            Ok(osmid) => {
                if grafo.osmid_para_id(osmid).is_some() {
                    return Ok(Some(osmid));
                } else {
                    println!("❌ OSMID {} não existe na malha de Ribeirão Preto.", osmid);
                }
            }
            Err(_) => {
                println!("❌ Formato numérico inválido. Exemplo: 259576101");
            }
        }

        print!("Deseja tentar novamente? [S/n]: ");
        io::stdout().flush()?;
        let mut retry = String::new();
        io::stdin().read_line(&mut retry)?;
        let r = retry.trim().to_lowercase();
        if r == "n" || r == "nao" || r == "não" {
            return Ok(None);
        }
    }
}

/// Executa os algoritmos A* e GBFS e exibe os resultados na formatação padrão do trabalho.
fn executar_e_exibir_buscas(
    grafo: &Grafo,
    origem: u64,
    destino: u64,
    peso_g: f64,
    peso_h: f64,
) -> io::Result<()> {
    if grafo.osmid_para_id(origem).is_none() {
        println!("❌ Erro: O cruzamento de origem {} não pertence ao grafo.", origem);
        return Ok(());
    }
    if grafo.osmid_para_id(destino).is_none() {
        println!("❌ Erro: O cruzamento de destino {} não pertence ao grafo.", destino);
        return Ok(());
    }

    // DESAFIO 1: Rota Ótima via A* (com pesos configurados)
    let t_inicio_astar = std::time::Instant::now();
    let res_a_star = grafo.a_star_ponderado(origem, destino, peso_g, peso_h);
    let tempo_a_star = t_inicio_astar.elapsed();

    // DESAFIO 2: Rota Expressa via GBFS
    let t_inicio_gbfs = std::time::Instant::now();
    let res_gbfs = grafo.gbfs(origem, destino);
    let tempo_gbfs = t_inicio_gbfs.elapsed();

    // Formatação Padrão do Trabalho - DESAFIO 1
    println!("------------------------------------------");
    println!("# RESULTADO DA BUSCA - DESAFIO 1 (ROTA ÓTIMA)");
    println!("------------------------------------------");
    println!("# Algoritmo utilizado: A* (A-Star)");
    println!("# Pesos de balanceamento: w_g = {:.2}, w_h = {:.2}", peso_g, peso_h);
    println!("# Origem (osmid): {}", origem);
    println!("# Destino (osmid): {}", destino);
    println!("------------------------------------------");
    if let Some(ref a_star) = res_a_star {
        println!("# Caminho encontrado (Qtd nós): {} cruzamentos", a_star.caminho.len());
        println!("# Distância da Rota: {:.2} metros", a_star.distancia_total);
        println!("# NÓS EXPANDIDOS: {}", a_star.nos_expandidos);
        println!("# Consumo de Memória (Pico): {}", formatar_memoria(a_star.memoria_pico_bytes));
        println!("# Tempo de CPU: {}", formatar_tempo(tempo_a_star));
    } else {
        println!("# A*: Nenhum caminho encontrado.");
    }
    println!("------------------------------------------\n");

    // Formatação Padrão do Trabalho - DESAFIO 2
    println!("------------------------------------------");
    println!("# RESULTADO DA BUSCA - DESAFIO 2 (ROTA EXPRESSA)");
    println!("------------------------------------------");
    println!("# Algoritmo utilizado: GBFS (Greedy Best-First Search)");
    println!("# Origem (osmid): {}", origem);
    println!("# Destino (osmid): {}", destino);
    println!("------------------------------------------");
    if let Some(ref gbfs) = res_gbfs {
        println!("# Caminho encontrado (Qtd nós): {} cruzamentos", gbfs.caminho.len());
        if let Some(ref a_star) = res_a_star {
            let diff_dist = ((gbfs.distancia_total - a_star.distancia_total) / a_star.distancia_total) * 100.0;
            let diff_exp = ((gbfs.nos_expandidos as f64 - a_star.nos_expandidos as f64) / a_star.nos_expandidos as f64) * 100.0;
            let diff_mem = ((gbfs.memoria_pico_bytes as f64 - a_star.memoria_pico_bytes as f64) / a_star.memoria_pico_bytes as f64) * 100.0;
            let speedup = tempo_a_star.as_secs_f64() / tempo_gbfs.as_secs_f64().max(1e-9);

            println!("# Distância da Rota: {:.2} metros ({:+.2}%)", gbfs.distancia_total, diff_dist);
            println!("# NÓS EXPANDIDOS: {} ({:+.2}%)", gbfs.nos_expandidos, diff_exp);
            println!("# Consumo de Memória (Pico): {} ({:+.2}%)", formatar_memoria(gbfs.memoria_pico_bytes), diff_mem);
            println!("# Tempo de CPU: {} ({:.1}x mais veloz que A*)", formatar_tempo(tempo_gbfs), speedup);

            if diff_dist <= 15.00 {
                println!("# Tolerância do Desafio: APROVADO ({:+.2}% <= limite de 15%)", diff_dist);
            } else {
                println!("# Tolerância do Desafio: EXCEDEU A MARGEM ({:+.2}% > limite de 15%)", diff_dist);
            }
        } else {
            println!("# Distância da Rota: {:.2} metros", gbfs.distancia_total);
            println!("# NÓS EXPANDIDOS: {}", gbfs.nos_expandidos);
            println!("# Consumo de Memória (Pico): {}", formatar_memoria(gbfs.memoria_pico_bytes));
            println!("# Tempo de CPU: {}", formatar_tempo(tempo_gbfs));
        }
    } else {
        println!("# GBFS: Nenhum caminho encontrado.");
    }
    println!("------------------------------------------\n");

    // Geração do mapa interativo Leaflet.js
    let caminho_mapa = "mapa_rotas.html";
    grafo.gerar_mapa_rotas(
        origem,
        destino,
        res_a_star.as_ref(),
        res_gbfs.as_ref(),
        caminho_mapa,
        false,
    )?;
    println!("# Mapa interativo salvo com sucesso em: {}\n", caminho_mapa);

    Ok(())
}

/// Formata uma duração de forma legível (µs, ms ou s).
fn formatar_tempo(duracao: Duration) -> String {
    let micros = duracao.as_micros();
    if micros < 1000 {
        format!("{} µs", micros)
    } else {
        let millis = duracao.as_secs_f64() * 1000.0;
        if millis < 1000.0 {
            format!("{:.2} ms", millis)
        } else {
            format!("{:.2} s", duracao.as_secs_f64())
        }
    }
}

/// Formata bytes em representação legível (B, KB ou MB).
fn formatar_memoria(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
