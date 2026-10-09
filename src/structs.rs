// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

//! # Módulo de Estruturas de Dados
//!
//! Este módulo define os tipos fundamentais e as estruturas de dados utilizadas para representar
//! a malha viária em memória com consumo mínimo de RAM, adotando o modelo
//! **CSR (*Compressed Sparse Row*)**.

use std::collections::HashMap;

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

/// Grafo Esparso Comprimido (*Compressed Sparse Row* - CSR) em arquitetura SoA (*Structure of Arrays*).
///
/// Adota o padrão SoA, separando os atributos das arestas em dois vetores contíguos paralelos
/// (`arestas_destino` e `arestas_distancia`), eliminando 100% de bytes de preenchimento (*padding*),
/// maximizando a localidade espacial de cache L1/L2 e permitindo serialização/desserialização direta de blocos.
#[derive(Debug)]
pub struct Grafo {
    /// Deslocamentos acumulados (*offsets*) para indexar os vizinhos de cada nó nos vetores CSR.
    ///
    /// Os vizinhos do nó `u` estão situados no intervalo de fatia:
    /// `offsets[u] as usize .. offsets[u + 1] as usize`.
    /// O array possui tamanho fixo de $N + 1$ elementos.
    pub offsets: Box<[u32]>,

    /// Vetor contíguo CSR contendo apenas os identificadores de destino dos vizinhos (u16 puro, sem padding).
    pub arestas_destino: Box<[NodeId]>,

    /// Vetor contíguo CSR contendo apenas as distâncias dos trechos viários (f64 puro, sem padding).
    pub arestas_distancia: Box<[f64]>,

    /// Coordenadas métricas planas de cada nó, indexadas diretamente pelo seu [`NodeId`].
    pub coordenadas: Box<[Coordenada]>,

    /// Mapeamento do identificador interno compacto ([`NodeId`]) para o `osmid` original do OpenStreetMap.
    pub id_para_osmid: Box<[u64]>,

    /// Tabela hash para conversão rápida de `osmid` externo para o índice interno compacto [`NodeId`].
    pub osmid_para_id: HashMap<u64, NodeId>,
}

impl Grafo {
    /// Converte um identificador externo `osmid` para o identificador numérico interno [`NodeId`].
    ///
    /// # Retorno
    ///
    /// Retorna `Some(NodeId)` se o nó pertencer à malha viária, ou `None` caso contrário.
    #[inline(always)]
    pub fn osmid_para_id(&self, osmid: u64) -> Option<NodeId> {
        self.osmid_para_id.get(&osmid).copied()
    }

    /// Converte um [`NodeId`] interno de volta para o identificador global `osmid` correspondente.
    ///
    /// # Parâmetros
    ///
    /// * `id` - Identificador interno no intervalo `0..total_nos()-1`.
    #[inline(always)]
    pub fn id_para_osmid(&self, id: NodeId) -> u64 {
        self.id_para_osmid[id as usize]
    }

    /// Obtém a coordenada métrica plana associada a um nó da malha viária.
    ///
    /// # Parâmetros
    ///
    /// * `id` - Identificador interno do nó desejado.
    #[inline(always)]
    pub fn coordenada(&self, id: NodeId) -> Coordenada {
        self.coordenadas[id as usize]
    }

    /// Retorna a fatia de nós de destino vizinhos do nó fornecido.
    ///
    /// Permite percorrer conexões com máxima eficiência de cache sem carregar distâncias desnecessariamente.
    #[inline(always)]
    pub fn vizinhos_destinos(&self, id: NodeId) -> &[NodeId] {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        &self.arestas_destino[inicio..fim]
    }

    /// Retorna a fatia de distâncias das arestas que partem do nó fornecido.
    #[inline(always)]
    pub fn vizinhos_distancias(&self, id: NodeId) -> &[f64] {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        &self.arestas_distancia[inicio..fim]
    }

    /// Retorna um iterador sobre os vizinhos do nó fornecido na forma de [`Aresta`].
    #[inline(always)]
    pub fn vizinhos(&self, id: NodeId) -> impl Iterator<Item = Aresta> + '_ {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        self.arestas_destino[inicio..fim]
            .iter()
            .zip(&self.arestas_distancia[inicio..fim])
            .map(|(&destino, &distancia)| Aresta::new(destino, distancia))
    }

    /// Retorna a quantidade total de nós (cruzamentos) presentes no grafo.
    #[inline(always)]
    pub fn total_nos(&self) -> usize {
        self.coordenadas.len()
    }

    /// Retorna a quantidade total de conexões direcionadas ativas no grafo.
    #[inline(always)]
    pub fn total_arestas(&self) -> usize {
        self.arestas_destino.len()
    }
}
