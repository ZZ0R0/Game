# Delta system — règles d’utilisation avec Knowledge

## Principe
Le moteur produit déjà des **deltas hiérarchiques** (added / modified / removed) qui permettent au client de rester synchronisé avec l’état serveur.

## Filtrage par connexion
Avant envoi, chaque delta est **filtré** par la **vue Player**:
- Si un champ n’est **pas autorisé** par le Knowledge du joueur, il est ignoré pour ce joueur.
- Si une entité devient visible/accessible, on envoie une **base initiale limitée** aux canaux autorisés, puis les deltas suivants.
- Si un accès est perdu (ex: relais coupé), on envoie des **tombstones** pour retirer les champs/canaux devenus interdits.

## AOI vs Knowledge
- **AOI**: décide ce qui est chargé physiquement (streaming géométrique).
- **Knowledge**: décide **quels champs** d’une entité donnée sont envoyés au client.
Les deux sont indépendants et complémentaires.

## Résilience
En cas de désynchronisation, recharger l’objet complet autorisé pour le joueur puis reprendre l’application des deltas.