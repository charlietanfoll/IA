// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

pub(crate) mod graph;
pub(crate) mod a_starr;
pub(crate) mod structs;
pub(crate) mod gbfs;
pub(crate) mod mapa;

use std::io::Error;

use graph::*;

fn main() -> Result<(), Error> {
    let ribeirao_graph = Grafo::new()?;
    let origem = 259576101;
    let destino = 259577028;

    println!("Executando buscas de {} para {}...\n", origem, destino);

    // DESAFIO 1: Rota Ótima via A*
    let resultado_a_star = ribeirao_graph.a_star(origem, destino);
    if let Some(ref res_a_star) = resultado_a_star {
        println!("------------------------------------------");
        println!("# RESULTADO DA BUSCA - DESAFIO 1 (ROTA ÓTIMA)");
        println!("------------------------------------------");
        println!("# Algoritmo utilizado: A* (A-Star)");
        println!("# Origem (osmid): {}", origem);
        println!("# Destino (osmid): {}", destino);
        println!("------------------------------------------");
        println!("# Caminho encontrado (Qtd nós): {} cruzamentos", res_a_star.caminho.len());
        println!("# Distância da Rota: {:.2} metros", res_a_star.distancia_total);
        println!("# NÓS EXPANDIDOS: {}", res_a_star.nos_expandidos);
        println!("------------------------------------------\n");
    } else {
        println!("A*: Nenhum caminho encontrado.\n");
    }

    // DESAFIO 2: Rota Expressa Subótima via GBFS
    let resultado_gbfs = ribeirao_graph.gbfs(origem, destino);
    if let Some(ref res_gbfs) = resultado_gbfs {
        println!("------------------------------------------");
        println!("# RESULTADO DA BUSCA - DESAFIO 2 (ROTA EXPRESSA)");
        println!("------------------------------------------");
        println!("# Algoritmo utilizado: GBFS (Greedy Best-First Search)");
        println!("# Origem (osmid): {}", origem);
        println!("# Destino (osmid): {}", destino);
        println!("------------------------------------------");
        println!("# Caminho encontrado (Qtd nós): {} cruzamentos", res_gbfs.caminho.len());

        if let Some(ref res_a_star) = resultado_a_star {
            let diff_dist = ((res_gbfs.distancia_total - res_a_star.distancia_total) / res_a_star.distancia_total) * 100.0;
            let diff_exp = ((res_gbfs.nos_expandidos as f64 - res_a_star.nos_expandidos as f64) / res_a_star.nos_expandidos as f64) * 100.0;
            println!("# Distância da Rota: {:.2} metros ({:+.2}%)", res_gbfs.distancia_total, diff_dist);
            println!("# NÓS EXPANDIDOS: {} ({:+.2}%)", res_gbfs.nos_expandidos, diff_exp);
        } else {
            println!("# Distância da Rota: {:.2} metros", res_gbfs.distancia_total);
            println!("# NÓS EXPANDIDOS: {}", res_gbfs.nos_expandidos);
        }

        println!("------------------------------------------");
    } else {
        println!("GBFS: Nenhum caminho encontrado.");
    }
    
    // Gera o mapa interativo HTML com A* (Roxo) e GBFS (Vermelho)
    let caminho_mapa = "mapa_rotas.html";
    ribeirao_graph.gerar_mapa_rotas(
        origem,
        destino,
        resultado_a_star.as_ref(),
        resultado_gbfs.as_ref(),
        caminho_mapa,
        false, // Deixamos false por enquanto para testes de terminal, ou true para abrir no browser
    )?;
    println!("\nMapa interativo salvo com sucesso em: {}", caminho_mapa);

    Ok(())
}
