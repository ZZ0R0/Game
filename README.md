Projet : Jeu clone de space engineers

Sandbox spatial d’ingénierie et survie.
Construction modulaire par blocs-grilles (vaisseaux, stations).
Planètes et astéroïdes voxel destructibles, minage et terraformation.
Physique newtonienne, énergie, ressources, production, logistique.
Dégâts structurels, pressurisation, maintenance.
Solo et multijoueur serveur dédié, coop et PvP.
Scripts et mods pour automatisation et contenu.

Instruction :
1) TOUJOURS, bien lire le repo et les readme pour savoir ce que contient les repo
2) les readme que écris doivent etre minimaux
3) les modifications que tu réalises doivent contenir le code fonctionnel le plus compact  et lisible possible, créer peu de fichiers différents, si possible utilise les fichiers déja présents si le code que tu veux ajouter n'est pas trop gros, sinon, créer un ficheir si il y a trop de code à ajouter, créer peu de code, mais toujours fonctionnel et d'un grade professionel
4) ne jamais créer de fichier ou de code de test, le seul test est de lancer le serveur puis le client
5) ne jamais uitliser d'emojis
6) ne jamais coder d'exemples qui ne représente pas l'entièreté du projet 
7) fais les choses commes elles serait faites par un developpeur professionel, ne fais pas de courts circuits dans tes raisonementsn le but est d'avoir le meilleur code possible, mais ne jamais oublier les instruction numéro 3

TODO :
0) ✅ FAIT - Le cube affiché est maintenant un light_armor_block appartenant à un LargeGrid créé par le serveur et rendu par le client
1) ✅ FAIT - Overlay console ajouté avec affichage des informations de jeu
2) ✅ FAIT - Compteur FPS affiché dans la console (environ 60 FPS)
3) ✅ FAIT - Position et orientation du joueur affichées dans la console
4) ajouter un mode plein écran avec F11






client du jeu "game_client"

s'occupe de 

1) fenètre de jeu
2) utilisation des fonctions haut niveau du renderer graphique du jeu
3) interractions réseau avec le serveur
3) logique interne du client



assets partagés du jeu "game_core"

s'occupe de 

1) toutes les structs poo de base du jeu
2) toutes les méthodes sur ces structs
3) toute la logique fondamentale du jeu

ne s'occupe pas de 

1) ggraphiques
2) réseau
3) code uniquement client
4) code uniquement serveur


gestion du protocole de communication client-serveur du jeu "game_protocol"

s'occupe de 

1) protocole quick entre le client et le serveur
2) reconnection du client
3) connections multi clients
4) selection des données à envoyer au client/ cahrgées par le client avec deltas de modifications minimaux


serveur du jeu "game_server"

s'occupe de 

1) engine du jeu (positions, données, logique)
2) utilisation des fonctions haut niveau du moteur phyisque du jeu
3) interractions réseau avec le client
3) logique interne du serveur