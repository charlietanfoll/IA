// Integrantes do Grupo:
// - Charlie TP. Lobo (Nº USP: 16827968)

use std::collections::HashMap;

struct caminho {
    origem: u32,
    destino: u32,
    custo: f32, // sujeira, nn gosto de numeros flutuantes...
    nome: String,
};

pub struct graph {
    nodes: HashMap<u32, Vec<caminho>>,
};
impl graph {};
