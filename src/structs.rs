// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

//! # Módulo de Estruturas de Dados
//!
//! Este módulo define os tipos fundamentais e as estruturas de dados utilizadas para representar
//! a malha viária em memória com consumo mínimo de RAM, adotando o modelo
//! **CSR (*Compressed Sparse Row*)**.

/// Identificador numérico interno compacto para nós (vértices) no intervalo `0..N-1`.
///
/// Como a malha viária de Ribeirão Preto contém 19.036 nós, o tipo primitivo `u16`
/// (que suporta até 65.535 valores) é suficiente, consumindo metade da memória de um `u32`
/// e um quarto da memória de um `u64`.
pub type NodeId = u16;

/// Posição métrica plana projetada em metros (UTM SIRGAS 2000 / 22S).
///
/// Permite o cálculo direto da distância euclidiana em metros para uso como heurística admissível no A*.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordenada {
    /// Coordenada UTM no eixo Este (X) em metros.
    pub x: f64,
    /// Coordenada UTM no eixo Norte (Y) em metros.
    pub y: f64,
}

impl Coordenada {
    /// Cria uma nova instância de [`Coordenada`].
    ///
    /// # Parâmetros
    ///
    /// * `x` - Coordenada UTM Este em metros.
    /// * `y` - Coordenada UTM Norte em metros.
    #[inline(always)]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calcula a distância euclidiana em linha reta (em metros) até outra coordenada.
    ///
    /// Esta função atua diretamente como a heurística admissível $h(n)$ para o algoritmo A\*,
    /// pois a linha reta euclidiana plana nunca superestima a distância real por malha viária.
    ///
    /// # Parâmetros
    ///
    /// * `outro` - A coordenada de destino contra a qual a distância será calculada.
    ///
    /// # Retorno
    ///
    /// Distância euclidiana euclidiana calculada em metros (`f64`).
    #[inline(always)]
    pub fn distancia_euclidiana(&self, outro: &Coordenada) -> f64 {
        let dx = self.x - outro.x;
        let dy = self.y - outro.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Representa uma conexão direcionada ponderada dentro da estrutura contígua CSR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aresta {
    /// Identificador compacto do nó de destino ([`NodeId`] no intervalo `0..N-1`).
    pub destino: NodeId,
    /// Custo de deslocamento ou extensão do trecho viário em metros.
    pub distancia: f64,
}

impl Aresta {
    /// Cria uma nova aresta ponderada direcionada.
    ///
    /// # Parâmetros
    ///
    /// * `destino` - Identificador do nó vizinho ([`NodeId`]).
    /// * `distancia` - Distância física do trecho em metros.
    #[inline(always)]
    pub const fn new(destino: NodeId, distancia: f64) -> Self {
        Self { destino, distancia }
    }
}

/// Estrutura para armazenar o resultado da execução de um algoritmo de busca.
#[derive(Debug, Clone)]
pub struct ResultadoBusca {
    /// Sequência ordenada de identificadores OpenStreetMap (`osmid`) da rota.
    pub caminho: Vec<u64>,
    /// Distância total percorrida pela rota em metros.
    pub distancia_total: f64,
    /// Total de nós retirados da fronteira para expansão de vizinhos.
    pub nos_expandidos: usize,
}

