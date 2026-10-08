use std::collections::HashMap;

/// Identificador numérico interno compacto para nós (0 a N-1).
/// Como a base possui 19.037 nós, o tipo `u16` (suporta até 65.535) economiza espaço em relação a `u32` ou `u64`.
pub type NodeId = u16;

/// Representa uma coordenada métrica plana (UTM em metros).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordenada {
    pub x: f64, // UTM_X_Este
    pub y: f64, // UTM_Y_Norte
}

impl Coordenada {
    /// Cria uma nova coordenada.
    #[inline(always)]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calcula a distância euclidiana em linha reta (em metros) até outra coordenada.
    /// Utilizada diretamente como a heurística admissível h(n) para o A*.
    #[inline(always)]
    pub fn distancia_euclidiana(&self, outro: &Coordenada) -> f64 {
        let dx = self.x - outro.x;
        let dy = self.y - outro.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Representa uma conexão direcionada no grafo comprimido.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aresta {
    /// Identificador do nó de destino (0..N-1)
    pub destino: NodeId,
    /// Custo/distância da aresta em metros
    pub distancia: f64,
}

impl Aresta {
    #[inline(always)]
    pub const fn new(destino: NodeId, distancia: f64) -> Self {
        Self { destino, distancia }
    }
}

/// Representação de Grafo Esparso Comprimido (CSR - Compressed Sparse Row).
/// 
/// Armazena todos os nós e arestas em blocos contíguos de memória (`Box<[T]>`),
/// minimizando alocações na heap, otimizando o uso do cache e permitindo acesso O(1).
#[derive(Debug)]
pub struct Grafo {
    /// Deslocamentos para os vizinhos de cada nó no vetor contíguo `arestas`.
    /// Os vizinhos do nó `u` estão na fatia `arestas[offsets[u]..offsets[u + 1]]`.
    /// Possui tamanho N + 1.
    pub offsets: Box<[u32]>,

    /// Conexões agrupadas e ordenadas por nó de origem.
    pub arestas: Box<[Aresta]>,

    /// Coordenadas de cada nó, indexadas diretamente pelo seu `NodeId` interno.
    pub coordenadas: Box<[Coordenada]>,

    /// Mapeamento do `NodeId` interno (índice 0..N-1) de volta para o `osmid` externo original.
    pub id_para_osmid: Box<[u64]>,

    /// Tabela de conversão do `osmid` externo original para o `NodeId` interno.
    pub osmid_para_id: HashMap<u64, NodeId>,
}

impl Grafo {
    /// Converte um `osmid` externo para o identificador interno `NodeId`.
    #[inline(always)]
    pub fn osmid_para_id(&self, osmid: u64) -> Option<NodeId> {
        self.osmid_para_id.get(&osmid).copied()
    }

    /// Converte um `NodeId` interno de volta para o seu `osmid` externo original.
    #[inline(always)]
    pub fn id_para_osmid(&self, id: NodeId) -> u64 {
        self.id_para_osmid[id as usize]
    }

    /// Obtém a coordenada métrica de um nó pelo seu `NodeId`.
    #[inline(always)]
    pub fn coordenada(&self, id: NodeId) -> Coordenada {
        self.coordenadas[id as usize]
    }

    /// Retorna a fatia de vizinhos do nó dado pelo seu `NodeId`.
    #[inline(always)]
    pub fn vizinhos(&self, id: NodeId) -> &[Aresta] {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        &self.arestas[inicio..fim]
    }

    /// Quantidade total de nós no grafo.
    #[inline(always)]
    pub fn total_nos(&self) -> usize {
        self.coordenadas.len()
    }

    /// Quantidade total de arestas direcionadas no grafo.
    #[inline(always)]
    pub fn total_arestas(&self) -> usize {
        self.arestas.len()
    }
}
