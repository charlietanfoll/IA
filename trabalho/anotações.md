# O que vou implementar?

## Modelar dados e preparações

- [ ] Modelar o Grafo
  Atributos que precisamos ter de forma abstrata:
  - Armazenar Nós
  - Armazenar suas Conexões (Bidirecionais)
  
  Métodos:
  - [ ] Get Caminhos dos Osmids Vizinhos

- [ ] Modelar a **Posição** e suas operações:

  Atributos da Posição:
  - [ ] Posição X
  - [ ] Posição Y
  - [ ] Caminhos/Arestas do Ponto

  Implementações da Posição:
  - [ ] Distancia heuclidiana entre outro ponto

- [ ] Modelar o **Caminho** e suas operações:

  Atributos:
  - [ ] Posição X
  - [ ] Posição Y
  - [ ] Distância/Peso da Aresta
  - [ ] Acho que só?

  Sem Implementações (por enquanto).

---

## A\* (A Estrela)

- [ ] Implementar a busca A\* com a possibilidade de definir o peso da heuristica e do caminho, mas por padrão, cada um pesa 0.5.

## GBFS (Greedy Best-First Search)