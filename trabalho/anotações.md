# O que vou implementar?

## Modelar dados e preparações

- [x] Modelar o Grafo (CSR - Compressed Sparse Row com arquitetura SoA em `src/structs.rs`)
  Atributos:
  - `offsets: Box<[u32]>` (deslocamento para os vizinhos de cada nó nos arrays contíguos)
  - `arestas_destino: Box<[NodeId]>` (identificadores de destino contíguos, sem padding)
  - `arestas_distancia: Box<[f64]>` (distâncias das arestas contíguas, sem padding)
  - `coordenadas: Box<[Coordenada]>` (posições dos nós indexadas por `NodeId`, `#[repr(C)]`)
  - `id_para_osmid: Box<[u64]>` (conversão do índice interno `NodeId` para `osmid` externo)
  - `osmid_para_id: HashMap<u64, NodeId>` (tabela de conversão do `osmid` externo para `NodeId` interno)

  Métodos implementados:
  - [x] `osmid_para_id(osmid: u64) -> Option<NodeId>`
  - [x] `id_para_osmid(id: NodeId) -> u64`
  - [x] `coordenada(id: NodeId) -> Coordenada`
  - [x] `vizinhos_destinos(id: NodeId) -> &[NodeId]`
  - [x] `vizinhos_distancias(id: NodeId) -> &[f64]`
  - [x] `vizinhos(id: NodeId) -> impl Iterator<Item = Aresta>`
  - [x] `total_nos() -> usize`
  - [x] `total_arestas() -> usize`

- [x] Modelar o identificador de Nó (`NodeId`):
  - `NodeId = u16` (índice compacto de 0 a N-1, cobrindo os 19.037 nós com metade da memória de um u32)

- [x] Modelar a **Coordenada** (`Coordenada`):
  Atributos:
  - [x] `x: f64` (UTM_X_Este em metros)
  - [x] `y: f64` (UTM_Y_Norte em metros)

  Implementações:
  - [x] `distancia_euclidiana(&self, outro: &Coordenada) -> f64` (cálculo em metros usado como heurística admissível h(n))

- [x] Modelar a **Aresta** (`Aresta`):
  Atributos:
  - [x] `destino: NodeId` (identificador do nó vizinho)
  - [x] `distancia: f64` (peso/distância do trecho em metros)

- [x] Parser e instanciação do Grafo a partir dos arquivos em `data/` (`src/graph.rs`):
  - [x] Leitura das coordenadas métricas (`data/coordenadas_finais.csv`)
  - [x] Leitura das conexões e montagem das fatias CSR bidirecionais (`data/arestas.csv`)
  - [x] Construção dos mapeamentos de `osmid` <-> `NodeId`
  - [x] Serialização e desserialização em arquivo binário `rp.graph` com Magic Bytes `RPGR`, versão 1 e byte sentinela `0xFF`

---

## A* (A Estrela)

- [ ] Implementar a busca A* com a possibilidade de definir o peso da heurística e do caminho, mas por padrão, cada um pesa 0.5.
- [ ] Garantir critério formal de contagem de nós expandidos.

## GBFS (Greedy Best-First Search)

- [ ] Implementar o algoritmo guloso focado no menor número de expansões para o Desafio 2.