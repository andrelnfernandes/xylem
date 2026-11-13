# Plano: menus que não cabem na largura

Origem: conversa de 2026-11-12. Com a janela estreita, `fit_menu` tira entradas
inteiras do fim da linha, sem avisar, e o "F9 next menu" (a tecla que troca de menu
nos terminais sem o protocolo do kitty) é o primeiro a sumir. Aprovado pelo autor.

## O que muda em `fit_menu`

1. **Encurtar as palavras por igual** (só nas linhas de palavras, `menu_words`): um
   limite de letras desce de um em um, para todas as palavras ao mesmo tempo, até a
   linha caber. As longas encolhem primeiro; as curtas ficam inteiras.
   - A letra destacada nunca é cortada (`eXecute` → `eXecu` → `eXe`).
   - Nas de tecla entre parênteses, o nome encolhe e a tecla fica:
     `· usage (W)` → `· usa(W)` → `· (W)`.
   - Sem ponto no fim; a segunda palavra sai inteira (`Compare branch` →
     `Compare`), só a primeira encolhe letra a letra.
   - Piso de 3 letras: abaixo disso, a etapa seguinte.
2. **Tirar entradas do fim com `…`**: o que ainda não cabe sai do fim, como hoje,
   mas um `…` fica no lugar, para mostrar que há mais.
3. **A última entrada das linhas de teclas fica** (F9, Esc): sai o que vem antes
   dela, e o `…` entra logo antes.

As dicas da segunda linha não são encurtadas: perdem partes inteiras (nos ` · `).

## Depois (não neste passo)

Um menu vertical com todos os itens, aberto quando há `…`: enquanto Ctrl ou Alt
está seguro (kitty) ou pelo F9.
