# PRD — Pois

> **Product Requirements Document — v0.2**
> Harness multi-agents multi-CLI, en Rust, distribué sous forme d'image Docker.
> Dernière mise à jour : 2026-04-24

---

## 1. Vision et positionnement

### 1.1 Pitch

**Pois** est un harness d'orchestration d'agents personnels qui permet de faire coexister plusieurs agents distincts (avec identité, mémoire et skills propres) dans un même environnement auto-hébergé, chacun pouvant s'exécuter via la CLI agentique de son choix (Goose, OpenCode, Crush) et changer de CLI ou de modèle en cours de conversation sans perdre le fil.

### 1.2 Inspirations et différenciation

Pois s'inspire de la philosophie Nanobot et OpenClaw — assistant personnel long-running, mémoire markdown, skills, cohabitation avec l'utilisateur sur la durée. La différence fondamentale est que Pois ne réimplémente pas sa propre boucle agentique. Il délègue cette boucle à des CLI agentiques externes existantes et apporte sa valeur sur la couche qui les entoure : gestion de l'identité persistante des agents, cloisonnement des mémoires, historique de conversation portable entre CLI, orchestration multi-agents, interface de contrôle unifiée, configuration déclarative.

Cette approche permet de bénéficier de la maturité des CLI agentiques de l'écosystème et d'éviter le travail d'implémentation et de maintenance d'une boucle agentique maison, tout en gardant le contrôle sur ce qui fait la personnalité d'un assistant : son identité, ce dont il se souvient, les canaux par lesquels il est joignable, la manière dont ses conversations sont préservées et restituées.

### 1.3 Utilisateur cible pour la v1

Développeur technique individuel qui souhaite disposer d'un atelier d'assistants personnels auto-hébergé, dans lequel plusieurs agents aux personnalités et missions différentes peuvent coexister. L'utilisateur choisit pour chaque agent la CLI la plus adaptée à ses tâches et peut expérimenter en changeant de CLI ou de modèle en cours de discussion. Accessible via une interface web et un canal WebSocket depuis ordinateur ou téléphone.

### 1.4 Positionnement

Pois n'est pas un coding assistant, ni un orchestrateur de CI, ni un framework de chaînage LLM. C'est un atelier d'agents personnels interchangeables, orienté vers la durée et la coexistence, pas vers l'exécution ponctuelle de tâches.

---

## 2. Scope

### 2.1 Inclus dans la v1

Harness Rust unique distribué comme image Docker, image basée sur Alpine avec couche de compatibilité gcompat pour les binaires glibc, architecture x86_64 uniquement.

Gestion multi-agents : N agents peuvent être créés et coexister, jusqu'à 3 peuvent tourner simultanément, chacun dispose d'une identité propre, d'un workspace isolé, d'une mémoire cloisonnée et de skills qui lui sont propres.

Capacité à changer de CLI et de modèle pour un agent donné, y compris en cours de conversation, en préservant l'historique et le workspace de cet agent.

Support de trois backends CLI en v1 : Goose, OpenCode et Crush. Cette diversité est voulue pour valider la robustesse de l'abstraction multi-backend sur des CLI aux architectures et aux formats d'I/O différents.

MCP unifié via un proxy HTTP agrégateur. Tous les serveurs MCP déclarés dans la configuration globale sont accessibles aux agents via un endpoint unique. Filtrage des outils par agent via allowlist et denylist par nom d'outil.

Mémoire sémantique gérée via MCP plutôt que par une intégration dédiée. Trois providers supportés en v1 : Honcho, mcp-memory-service, et le server-memory officiel. Le provider est choisi globalement par l'utilisateur dans un slot dédié. Pois se charge d'expanser ce slot en instances cloisonnées par agent, de manière invisible pour l'utilisateur.

Canal d'interaction unique par WebSocket, avec enveloppe JSON typée et streaming des events de la CLI vers le client au fil de l'eau.

Interface web construite avec HTMX et Pico.css, servant à configurer Pois, créer et gérer les agents, chatter avec un agent spécifique, visualiser l'état du système.

Scheduler cron permettant de déclencher des turns d'agents selon une planification déclarative.

Configuration canonique au format YAML, avec expansion des variables d'environnement pour les secrets.

Persistance mixte : base SQLite pour les données structurées (agents, sessions, messages, events, sessions UI), fichiers plats markdown pour les éléments lisibles par les CLI (AGENTS.md, soul.md, skills).

Authentification par session cookie avec mot de passe unique défini par variable d'environnement.

Gestion runtime des CLI via mise embarqué dans l'image, permettant à l'utilisateur d'ajouter de nouvelles CLI sans rebuilder l'image.

### 2.2 Exclu de la v1, reporté en v2

Canaux Telegram, WhatsApp, Discord ou autres.

Heartbeat (réveils proactifs des agents sans déclencheur utilisateur).

Subagents et background tasks longs asynchrones.

Multi-utilisateur, authentification par utilisateur distinct, gestion de rôles.

Fonctionnalités de self-improvement et de feedback loop.

Migrations automatiques de configuration entre versions majeures.

Support multi-architecture (arm64).

Previews automatiques par pull request et releases entièrement automatisées.

Tests automatisés de l'interface web.

Site de documentation externe avec framework statique.

Migration automatique entre providers de mémoire (un changement de provider entraîne la perte de la mémoire existante, c'est accepté).

---

## 3. Architecture fonctionnelle

### 3.1 Vue d'ensemble

Pois est une application Rust unique qui tourne dans un container. Elle expose une interface web et un canal WebSocket. Elle orchestre des subprocess CLI (Goose, OpenCode, Crush) qui incarnent la boucle agentique de chaque agent. Elle s'appuie sur un proxy MCP externe embarqué dans le même container pour agréger les serveurs MCP. Elle persiste son état dans un volume monté.

L'ensemble fonctionne mono-utilisateur. L'utilisateur se connecte à l'interface web via un mot de passe, gère ses agents, leur envoie des messages, et voit leurs réponses en temps réel.

### 3.2 Cycle de vie d'un message utilisateur

L'utilisateur, depuis l'interface web ou un client WebSocket, envoie un message à un agent spécifique. Le harness authentifie la requête via le cookie de session, puis transmet l'ordre au composant responsable de l'orchestration des runs d'agents.

Ce composant vérifie que l'agent n'est pas déjà en train de traiter un message (exclusivité par agent), puis vérifie qu'il reste de la capacité dans l'enveloppe globale de concurrence (trois agents en parallèle au maximum). Si ces conditions sont remplies, il acquiert les verrous nécessaires et déclenche l'exécution.

L'exécution consiste à reconstituer l'historique canonique de la conversation depuis la base, à le sérialiser sous forme de transcript lisible, à écrire ce transcript dans un fichier éphémère propre à ce run, puis à invoquer la CLI active de l'agent en lui passant ce transcript et le nouveau message. La CLI est lancée dans le workspace de l'agent comme répertoire courant, de sorte qu'elle voit nativement les fichiers d'instructions (AGENTS.md, soul.md) et les skills de l'agent.

Pendant l'exécution de la CLI, une tâche dédiée lit en continu sa sortie standard, parse les events qu'elle émet (appels d'outils, réponses du modèle, fin de turn), les normalise vers un format canonique, les persiste en base, et les diffuse en temps réel aux clients abonnés à cet agent via WebSocket. L'utilisateur voit ainsi l'agent travailler.

Lorsque la CLI termine son turn, le harness reconstruit le message final de l'assistant à partir des events, l'insère dans l'historique canonique, libère les verrous et la capacité, nettoie les fichiers éphémères. Le client reçoit un event de fin de réponse.

### 3.3 Changement de CLI en cours de conversation

L'utilisateur peut, via l'interface, demander qu'un agent change de CLI (par exemple passer de Goose à OpenCode) ou de modèle. Cette action est refusée si l'agent est en train de traiter un message. Une fois acceptée, elle met à jour la CLI active et le modèle actif de l'agent en base. Le prochain message envoyé à cet agent utilisera la nouvelle configuration.

La continuité de la conversation est assurée par le fait que l'historique est géré par Pois au format canonique indépendant des CLI. Lorsque la nouvelle CLI est invoquée, elle reçoit le même transcript que la précédente aurait reçu, simplement sérialisé et injecté de la manière qui lui est propre. Le workspace est également partagé : tous les fichiers produits par la CLI précédente restent accessibles à la nouvelle.

### 3.4 Gestion MCP et abstraction de la mémoire

Tous les serveurs MCP de configuration standard sont déclarés dans la configuration globale par l'utilisateur. Un proxy MCP tourne comme sidecar dans le container. Il agrège tous ces serveurs et les expose aux CLI via un endpoint HTTP unique, avec préfixage des noms d'outils pour éviter les collisions.

La mémoire des agents est traitée spécifiquement comme un concept produit de premier niveau, distinct des MCP génériques. L'utilisateur ne configure pas N serveurs MCP de mémoire ; il choisit un provider de mémoire global parmi une liste de candidats supportés, configure ses credentials une seule fois. Pois se charge en coulisses de générer autant d'instances MCP de ce provider qu'il y a d'agents, chacune configurée pour isoler la mémoire dans un namespace dédié à l'agent concerné.

Chaque agent voit donc, dans ses outils disponibles, des outils mémoire qui lui sont exclusifs, et aucun autre agent ne peut accéder à sa mémoire. En v1, le préfixage des outils mémoire conserve visiblement le nom de l'agent (par exemple `memory_community_manager_search` plutôt qu'un simple `memory_search`) ; l'utilisateur ne le voit pas car il n'intervient qu'en interne dans la communication entre la CLI et le proxy. Le renommage cosmétique vers un nom uniforme par agent pourra être envisagé après v1 si un besoin concret émerge.

### 3.5 Propagation des changements de configuration

Certaines modifications de configuration nécessitent un redémarrage du proxy MCP pour être prises en compte : création ou suppression d'un agent (car la liste d'instances mémoire change), changement du provider de mémoire, ajout ou suppression d'un serveur MCP global.

Pour éviter de couper des conversations en cours, l'interface grise le bouton de prise en compte tant qu'au moins un agent est en train de traiter un message. L'utilisateur voit immédiatement pourquoi l'action est temporairement indisponible. Quand tous les agents sont au repos, le bouton redevient actif. Le redémarrage du proxy est rapide (typiquement une à deux secondes) et les nouveaux runs sont brièvement mis en attente pendant cette fenêtre.

### 3.6 Agents et isolation

Chaque agent est un objet de première classe dans Pois. Il possède un nom unique, une description, une identité décrite en markdown (soul.md), des instructions durables (AGENTS.md), un ensemble de skills, une configuration propre (CLI par défaut, modèle par défaut, filtres d'outils, seuil de compacting). Son workspace est un répertoire dédié dans le volume, qui devient le répertoire de travail des CLI lancées pour son compte.

L'isolation entre agents est stricte. Aucun agent ne peut lire le workspace ou la mémoire d'un autre. Cela simplifie le modèle mental de l'utilisateur et évite les fuites contextuelles inattendues. Si un jour l'utilisateur a besoin de partager explicitement des éléments entre agents, cela sera traité en v2 par un mécanisme opt-in dédié.

### 3.7 Compacting de l'historique

Les conversations longues finissent par dépasser la fenêtre de contexte du modèle actif. Pois surveille la taille cumulée de l'historique d'un agent, exprimée en tokens estimés. Lorsque cette taille dépasse un seuil configurable (par défaut 80% de la fenêtre du modèle actif), Pois déclenche un compacting avant d'invoquer la CLI : les messages les plus anciens sont remplacés par un résumé généré par un appel LLM direct, et c'est la version compactée qui est injectée au run suivant.

L'historique intégral reste préservé en base pour permettre le replay, le debug et d'éventuelles consultations ultérieures ; c'est uniquement le transcript injecté à la CLI qui est compacté.

### 3.8 Concurrence et capacité

Pois limite le nombre de runs de CLI simultanés à trois, pour éviter la saturation mémoire et les rate limits côté provider LLM. Au-delà, les demandes sont mises en file d'attente FIFO et traitées à mesure que la capacité se libère. Un agent donné ne peut pas traiter deux messages en parallèle : si un message est déjà en cours sur cet agent, un nouveau message est mis en attente même s'il reste de la capacité globale.

L'utilisateur voit dans l'interface, pour chaque agent, son état (repos, en attente de capacité, en cours d'exécution) et pour le système global, le nombre de runs actifs sur les trois disponibles.

### 3.9 Canal WebSocket et interface web

L'interface web et le canal WebSocket partagent la même authentification et le même serveur. Un endpoint WebSocket unique accepte toutes les interactions avec tous les agents, avec routage par identifiant d'agent dans chaque message. Les events des runs sont diffusés aux clients abonnés à chaque agent, de manière indépendante.

Au reconnect d'un client, Pois lui envoie automatiquement les events récents des agents actuellement en cours d'exécution pour qu'il se remette à jour rapidement. Un endpoint de replay plus ancien est également disponible pour les cas où l'utilisateur veut consulter l'historique d'un run passé.

---

## 4. Décisions techniques

### 4.1 Plateforme et langage

Langage Rust en édition stable récente. Runtime asynchrone Tokio multi-thread, avec un runtime unique partagé par tous les composants (serveur web, orchestrateur de runs, parseurs d'events, scheduler). Framework web Axum pour le serveur HTTP et WebSocket. Templating Askama pour le rendu des pages HTMX, compile-time type-safe, erreurs détectées à la compilation.

### 4.2 Persistance

Base SQLite embarquée, un seul fichier dans le volume, accédée via sqlx avec vérification de requêtes à la compilation. Migrations versionnées appliquées automatiquement au démarrage.

La base stocke les agents, les sessions de conversation, les messages au format canonique, les events normalisés des CLI, les sessions d'interface web pour l'authentification, l'état connu des CLI et des serveurs MCP.

Les fichiers plats dans le workspace de chaque agent stockent ce qui doit être lisible nativement par les CLI : instructions (AGENTS.md), identité (soul.md, identity.md), configuration agent (agent.yaml), skills (répertoire skills avec fichiers SKILL.md).

Les transcripts éphémères utilisés pour injecter l'historique aux CLI lors d'un run vivent dans un sous-répertoire dédié du workspace de l'agent et sont nettoyés après le run.

### 4.3 Format canonique de l'historique

L'historique canonique adopte le format de messages Anthropic : rôles user et assistant, contenu composé de blocs typés (texte, appel d'outil, résultat d'outil). Ce choix est motivé par la richesse expressive du format, sa bonne couverture des appels d'outils et de leurs résultats, et la facilité de traduction vers les formats attendus par les CLI (qui préfèrent in fine un prompt texte construit depuis cette représentation).

L'injection de cet historique dans une CLI passe par la sérialisation en markdown lisible d'un transcript, écrit dans un fichier éphémère et passé à la CLI via son mécanisme d'entrée (fichier d'instructions ou stdin selon la CLI). Le fichier AGENTS.md du workspace contient les instructions durables de l'agent et n'est jamais touché par le mécanisme d'injection d'historique.

### 4.4 Authentification

Mot de passe unique défini par variable d'environnement au démarrage du container. Au login, le mot de passe saisi est comparé par argon2 contre le hash du mot de passe de référence calculé au démarrage. Si la comparaison réussit, un token de session aléatoire est généré et son hash est persisté en base avec date de création, date d'expiration et user-agent du client. Le token en clair est envoyé au navigateur dans un cookie sécurisé, accessible uniquement par le serveur, envoyé uniquement en HTTPS en production, avec restriction SameSite stricte.

À chaque requête authentifiée, le middleware récupère le token du cookie, calcule son hash, et cherche une session correspondante non expirée. Si trouvée, la requête est autorisée. Le logout supprime la session en base, invalidant immédiatement le cookie.

Cette approche permet de révoquer une session à tout moment, de lister les sessions actives de l'utilisateur dans l'interface, et de wipe toutes les sessions en cas de changement du mot de passe de référence. Le surcoût d'une requête SQLite par action authentifiée est négligeable à l'échelle mono-utilisateur.

### 4.5 Configuration

La configuration canonique est un fichier YAML unique. Les valeurs sensibles (clés d'API, mot de passe UI) ne sont jamais stockées en clair dans ce fichier : elles sont référencées via la syntaxe d'expansion de variables d'environnement, résolues au chargement. Si une variable référencée n'est pas définie au démarrage, Pois refuse de démarrer avec un message explicite.

La configuration comprend plusieurs sections : providers LLM et leurs credentials, configuration du serveur web, authentification UI, liste des CLI disponibles avec leur spécification mise, liste des serveurs MCP globaux, configuration du proxy MCP, slot dédié à la mémoire avec sélection du provider, configuration du scheduler, paramètres de compacting par défaut.

Chaque agent dispose également d'un fichier de configuration propre dans son workspace, qui contient sa CLI par défaut, son modèle par défaut, son seuil de compacting, ses filtres d'outils (allowlist et denylist), et toute surcharge éventuelle des paramètres globaux.

### 4.6 Mémoire — Providers supportés

Pois supporte trois providers de mémoire en v1. L'utilisateur en choisit un seul à la fois au niveau global ; changer de provider entraîne la perte de la mémoire accumulée précédemment, ce qui est accepté et documenté.

**Honcho** via son serveur MCP hébergé. Provider riche avec concept de peers, workspaces, raisonnement cross-session. Authentification par clé d'API. Isolation par agent via l'en-tête X-Honcho-Workspace-ID généré automatiquement à partir du nom de l'agent. Mode hébergé par défaut, possibilité de self-host reportée.

**mcp-memory-service** projet open-source qui expose une mémoire persistante avec recherche sémantique via MCP. Auto-hébergé, données stockées dans le volume Pois. Isolation par agent via instance dédiée par agent avec chemins de stockage distincts.

**server-memory** (le serveur officiel du projet Model Context Protocol). Mémoire sous forme de knowledge graph persisté en fichier JSONL local. Simple, pas de dépendance externe, pas de recherche sémantique avancée mais adapté aux cas d'usage factuels. Isolation par agent via chemins de fichiers distincts.

L'utilisateur voit dans l'interface un sélecteur de provider clair, avec un descriptif de chacun. Quand il change de provider, une confirmation explicite avertit de la perte de mémoire et propose un export JSON des mémoires actuelles pour archive (export brut, non ré-importable automatiquement — l'utilisateur peut le consulter s'il veut).

### 4.7 Providers LLM

Un seul provider LLM supporté en v1 : OpenRouter. C'est le provider par défaut, utilisé par toutes les CLI backends via leur configuration propre. Pois injecte la clé d'API OpenRouter comme variable d'environnement à chaque subprocess CLI.

Pois effectue lui-même quelques appels LLM directs en dehors des CLI, notamment pour le compacting d'historique. Ces appels passent aussi par OpenRouter, avec un modèle configurable (un modèle rapide et bon marché par défaut, l'utilisateur peut override).

### 4.8 Runtime container et distribution

Runtime container : Podman en développement local, Docker en production sur Railway. Les deux sont compatibles sur le Dockerfile standard que Pois produit. L'image est construite avec Docker mais fonctionne avec Podman en local grâce à cette compatibilité.

Image de base Alpine, avec la couche de compatibilité gcompat installée pour permettre l'exécution de binaires glibc téléchargés dynamiquement par mise. Si des blocages récurrents apparaissent sur plus de deux CLI populaires pendant le développement, bascule prévue vers debian-slim.

Architecture x86_64 uniquement en v1. Les développeurs sur Apple Silicon acceptent l'émulation locale lente via QEMU ; un support multi-architecture est reporté.

L'image embarque : le binaire Rust de Pois compilé en statique musl, mise pour la gestion des CLI, le proxy MCP (TBXark/mcp-proxy) comme binaire autonome, Node.js et Python avec leurs gestionnaires de packages (npm et uv) pour permettre l'exécution de serveurs MCP stdio externes.

L'image est publiée sur GitHub Container Registry en public, avec plusieurs tags : latest pour la dernière release stable, des tags sémantiques par version (0.1.0, 0.2.0, etc.) pour pinning, et un tag edge mis à jour automatiquement depuis la branche principale.

### 4.9 Assets web et dépendances front-end

HTMX et Pico.css sont chargés via CDN depuis des URLs avec version pinnée. Pois ne tourne pas hors ligne puisqu'il appelle OpenRouter pour chaque run, donc la dépendance à Internet sortant est déjà un prérequis. Le CDN évite le vendoring et simplifie les mises à jour. Un fichier CSS custom minimal, vendored dans l'image, permet les quelques overrides de style spécifiques à Pois.

### 4.10 Logging et observabilité

Le logging est hybride. Les events métier significatifs (démarrage d'agent, envoi de message, appel d'outil, fin de run, changement de CLI, dépassement de seuil de compacting, erreur MCP, rate limit provider) sont persistés comme events structurés en base, consultables dans l'interface avec filtres par agent, par type et par fenêtre temporelle.

En parallèle, les logs techniques du harness (démarrage, erreurs internes, requêtes HTTP, upgrade WebSocket) sont écrits dans un fichier log rotatif dans le volume, et également émis vers la sortie standard du container pour capture par l'infrastructure (Railway). Le niveau de verbosité est configurable par variable d'environnement.

L'interface expose une page "Events" pour les events métier avec filtres, et une page "Logs" qui affiche les dernières lignes du fichier log technique.

### 4.11 Tests

La philosophie de tests est "chemins critiques, pas de couverture exhaustive", cohérente avec le niveau "artisanal publié" visé et le temps à y consacrer. Les tests unitaires sont écrits au fil de l'eau dans les modules concernés, principalement sur la logique pure (parsing de config, sérialisation de transcript, validation, merge de configuration globale et agent). Les tests d'intégration couvrent les parcours critiques : authentification complète, cycle de vie d'un agent avec un backend simulé, changement de CLI en cours de conversation, déclenchement de compacting, persistance après redémarrage simulé, application correcte des filtres d'outils par agent.

Un backend simulé implémente l'interface d'abstraction CLI avec des scénarios programmables en fichiers fixtures, permettant de tester l'orchestration sans invoquer de vraies CLI ni consommer de tokens. Un mock HTTP intercepte les appels à OpenRouter et aux serveurs MCP HTTP pour les tests qui nécessitent ces interactions.

Les tests end-to-end avec les vraies CLI et de vrais appels LLM sont exécutés manuellement avant chaque release significative, selon une checklist documentée.

### 4.12 Versioning et release

Versioning sémantique en série 0.x.y jusqu'à stabilisation de la surface de configuration. En pré-1.0, les breaking changes sont tolérés dans la série 0.x. Le passage à 1.0.0 signalera une stabilité de contrat.

Trois tags d'image cohabitent : latest pointant sur la dernière release stable taguée, les tags sémantiques pour chaque release, et edge mis à jour automatiquement depuis la branche principale pour les utilisateurs qui veulent suivre le développement.

Le changelog est maintenu manuellement, avec des entrées concises par version signalant les additions, modifications, corrections et breaking changes. Un champ de version est présent dans le fichier de configuration YAML pour permettre à Pois de valider au démarrage qu'il comprend bien le format, et refuser de démarrer avec un message clair si la version est incompatible.

Les migrations de schéma de base sont appliquées automatiquement au démarrage. Les migrations de format de configuration ne sont pas automatisées en v1 ; un breaking change majeur demandera à l'utilisateur de régénérer sa configuration.

### 4.13 Documentation

La documentation est distribuée en deux niveaux. Un README concis à la racine du repo présente le projet, le quickstart et renvoie vers la documentation détaillée. Un dossier de documentation contient des fichiers markdown thématiques : référence de la configuration globale et par agent, guide de création d'agents, guide des backends CLI supportés et de l'ajout de nouvelles CLI, guide des serveurs MCP, guide des providers de mémoire, troubleshooting, architecture, cookbook de recettes concrètes, checklist de release à l'usage du mainteneur.

Un sous-dossier d'exemples contient deux à trois agents prêts à être copiés par l'utilisateur pour démarrer rapidement, chacun avec son soul.md, son agent.yaml et un bref guide d'usage.

### 4.14 Onboarding utilisateur

Au premier démarrage, quand aucun agent n'existe, l'interface redirige l'utilisateur vers un assistant de création en plusieurs étapes. L'assistant lui demande le nom de son premier agent, une courte description, la CLI qu'il veut utiliser par défaut (choisie dans la liste des CLI installées via mise), le modèle par défaut. Il génère le workspace de l'agent avec des fichiers AGENTS.md et soul.md initiaux, enregistre l'agent en base et redirige vers la vue de conversation avec ce nouvel agent.

---

## 5. Interface utilisateur

### 5.1 Vue d'ensemble

L'interface web est construite en HTMX avec Pico.css comme base de styling. Elle est volontairement simple et fonctionnelle, pas orientée vitrine. L'utilisateur ne s'attend pas à une expérience produit polie mais à un outil technique efficace.

### 5.2 Pages principales

**Page de login** affichée quand l'utilisateur n'est pas authentifié. Un champ mot de passe, un bouton de soumission, pas de création de compte.

**Tableau de bord** affichant la liste des agents avec leur état (repos, en cours, en file d'attente), la CLI et le modèle actifs de chacun, et l'utilisation globale des 3 slots de concurrence. Accès rapide à chaque agent.

**Vue d'agent** pour une conversation dédiée. En haut, le nom et la description de l'agent, les sélecteurs de CLI et modèle (grisés quand l'agent est en cours). Au centre, le fil de conversation avec les messages user et assistant, les appels d'outils affichés de manière lisible, les events en cours streamés en temps réel. En bas, une zone de saisie pour envoyer un nouveau message.

**Page de configuration globale** pour éditer les sections de la configuration canonique : providers LLM, providers MCP, provider de mémoire (avec sélecteur clair de Honcho, mcp-memory-service ou server-memory), CLI disponibles, paramètres du serveur, planification cron. Chaque modification est validée syntaxiquement avant sauvegarde. Le bouton de prise en compte des modifications nécessitant un redémarrage du proxy MCP est grisé tant qu'au moins un agent est en cours d'exécution, avec un message explicatif.

**Page d'édition d'agent** pour modifier la configuration d'un agent existant : description, CLI et modèle par défaut, seuil de compacting, allowlist et denylist d'outils, lien vers l'édition directe de son AGENTS.md et son soul.md.

**Page Events** affichant les events métier filtrables par agent, par type d'event et par fenêtre temporelle. Utile pour comprendre ce qu'il s'est passé récemment ou déboguer une session.

**Page Logs** affichant les dernières lignes du log technique pour diagnostic.

**Page Sessions** listant les sessions d'authentification actives, permettant à l'utilisateur de révoquer des sessions ou de se déconnecter partout.

### 5.3 Interactions temps réel

Les vues d'agent et le tableau de bord s'actualisent en temps réel via WebSocket. Un agent qui passe d'un état à un autre se met à jour immédiatement chez tous les clients connectés. Les messages et events des runs sont streamés au fur et à mesure, avec une expérience comparable à celle d'un terminal affichant une CLI interactive.

### 5.4 Contraintes mobile

L'interface est utilisable sur mobile de manière fonctionnelle mais pas optimisée. Pico.css est nativement responsive et les vues principales restent lisibles sur petit écran. Aucun effort particulier d'ergonomie mobile n'est fait en v1 ; un polish mobile viendra ultérieurement si l'usage le justifie.

---

## 6. Structure du projet

### 6.1 Organisation du code

Le code est organisé par domaine fonctionnel : configuration, authentification, gestion des agents, orchestrateur de runs, abstraction des backends CLI, sérialisation des transcripts, intégration MCP, wrapper mise, canal WebSocket, serveur web et templates, accès base de données, système d'events, compacting, scheduler.

L'abstraction des backends est une pièce centrale : une interface commune décrit ce qu'un backend CLI doit fournir (comment invoquer la CLI, comment générer sa configuration éphémère, comment parser sa sortie en events canoniques, comment extraire un message final). Trois implémentations existent en v1 pour Goose, OpenCode et Crush, plus une implémentation simulée pour les tests.

### 6.2 Arborescence du workspace

Le volume de Pois est organisé en trois branches. Une branche "data" contient la configuration globale, la base SQLite, et un sous-répertoire par agent avec ses fichiers markdown, sa configuration, ses skills et ses transcripts éphémères. Une branche "cache" contient les installations des CLI gérées par mise et les caches du proxy MCP, tout ce qui est régénérable et n'a pas besoin d'être backupé. Une branche "logs" contient les fichiers log rotatifs.

Cette organisation permet à l'utilisateur de backuper uniquement "data" pour sauvegarder son état utile, et de régénérer "cache" et "logs" à volonté.

### 6.3 CI

Les vérifications de continuous integration s'exécutent sur GitHub Actions et comprennent le formatage du code Rust, le linting strict, l'exécution de la suite de tests, la construction de l'image Docker et son push sur le registry. Un workflow de release séparé se déclenche sur les tags de version et produit l'image taguée correspondante.

---

## 7. Chiffrage

### 7.1 Hypothèses

Les estimations ci-dessous supposent un développeur débutant en Rust travaillant majoritairement avec assistance IA, avec pour objectif le niveau "artisanal publié" : image Docker fonctionnelle, documentation honnête, tests sur les chemins critiques, gestion d'erreurs raisonnable, pas de polish excessif. Les heures sont des heures de travail effectif, pas du temps calendaire.

### 7.2 Estimation par domaine

| Domaine | Heures estimées |
|---|---|
| Configuration (canonique globale et par agent, validation, UI d'édition) | 18-25h |
| Authentification (login, sessions, middleware, authentification WebSocket, page sessions actives) | 8-12h |
| Couche base de données (sqlx, migrations, queries, modèles) | 12-18h |
| Cycle de vie des agents (CRUD, state machine, orchestrateur, sémaphore) | 25-35h |
| Abstraction des backends CLI (interface commune plus trois implémentations Goose, OpenCode, Crush) | 60-85h |
| Sérialisation des transcripts et gestion de l'historique | 8-12h |
| Intégration MCP (génération de config proxy, gestion de cycle de vie, filtrage par agent) | 10-15h |
| Abstraction mémoire (templates de providers, expansion par agent, UI) | 12-18h |
| Intégration mise runtime | 8-12h |
| Compacting | 6-10h |
| Bus d'events (broadcast, parsing, normalisation, persistance) | 15-20h |
| Canal WebSocket (serveur, protocole, replay) | 15-20h |
| Scheduler cron | 8-12h |
| Interface HTMX (Axum, Askama, toutes les pages) | 30-40h |
| Logging et observabilité | 8-12h |
| Dockerfile, entrypoint, CI | 10-15h |
| Tests (infrastructure de mock, tests d'intégration critiques) | 23-32h |
| Documentation complète | 12-18h |
| Assistant d'onboarding et agents d'exemple | 7-11h |
| Marge pour imprévus, debug Alpine-gcompat, intégrations amont | 20-30h |

**Total v1 estimé : 315 à 450 heures.**

En side project à environ 12 heures effectives par semaine, cela représente entre 6 et 9 mois calendaires de travail.

### 7.3 Option MVP v0.5

Une version 0.5 réduite permet de valider le concept plus rapidement avant d'investir dans toute l'abstraction : un seul backend CLI (Goose uniquement), un seul agent, pas de changement de CLI en cours de discussion, interface minimaliste, compacting basique par fenêtre glissante sans résumé LLM, pas de scheduler cron, configuration manuelle sans assistant.

Estimation v0.5 : 140 à 190 heures, soit 3 à 4 mois en side project. Objectif : valider que le pipeline Rust vers mise vers Goose vers proxy MCP vers OpenRouter vers mémoire MCP fonctionne end-to-end avant d'engager le travail d'abstraction multi-backends et multi-agents.

---

## 8. Risques et mitigations

| Risque | Impact | Mitigation |
|---|---|---|
| Incompatibilités musl/gcompat avec les binaires CLI téléchargés dynamiquement | Bloquant pour une ou plusieurs CLI | Fallback vers debian-slim prévu, évaluation précoce sur chaque CLI |
| Mode non-interactif de Crush moins mature que les deux autres CLI | Events manqués, parsing fragile | Backend simulé en premier, tests réels sur Crush précoces, retour possible vers Goose et OpenCode seulement si Crush ne convient pas |
| Changements breaking amont dans les CLI | Régression silencieuse après update | Versions pinnées dans mise, tests smoke en CI, vérification manuelle avant chaque release |
| Bugs ou abandon de TBXark/mcp-proxy | Bloquant pour l'agrégation MCP | Alternatives identifiées (mcp-rust-proxy, MetaMCP), couche d'abstraction côté Pois qui permet de swapper |
| Changements d'API ou downtime des providers de mémoire MCP | Perte de fonctionnalité mémoire pour un provider | Trois providers supportés en v1 permet de swapper, mode "mémoire désactivée" reste fonctionnel |
| Rate limits OpenRouter | Runs qui échouent | Retry avec backoff exponentiel, affichage clair dans l'UI, documentation troubleshooting |
| Bugs de concurrence (races, deadlocks) | Crash ou blocage | Tests d'intégration sur les parcours concurrents, tracing verbeux en développement |
| Sur-ingénierie de l'abstraction backend | Retard et refacto | Démarrer avec un seul backend concret, abstraire une fois que le pipeline fonctionne |
| Sous-estimation du travail UI | Retard | Accepter une UI fonctionnelle mais peu polie en v1, itérer ensuite |
| Perte de mémoire au changement de provider | Frustration utilisateur | Documenté clairement, export brut proposé avant changement |

---

## 9. Roadmap post-v1

### 9.1 Itérations mineures v1.x

Amélioration du compacting avec tokenizer réel et stratégies hiérarchiques. Support d'un heartbeat permettant aux agents de se réveiller proactivement sur timer. Support des subagents pour des tâches longues asynchrones. Ajout du canal Telegram. Métriques basiques d'usage (nombre de runs, tokens consommés par agent, durée moyenne des turns).

### 9.2 Version 2.0

Support de canaux supplémentaires (Discord, Slack, WhatsApp). Passage multi-utilisateur avec isolation des workspaces et gestion d'authentification par utilisateur distinct. Support de l'architecture arm64. Documentation enrichie avec site statique. Marketplace ou bibliothèque de skills et templates d'agents. Proxy MCP maison en Rust avec rechargement à chaud pour éliminer les redémarrages lors des changements de configuration. Migration automatisable entre providers de mémoire quand c'est techniquement faisable.

---

## 10. Définition de "done" pour la v1

La v1 est considérée terminée lorsque les critères suivants sont remplis de bout en bout.

L'image Docker est publiée sur GitHub Container Registry et fonctionne identiquement par un `pull` puis `run` sur un laptop et sur Railway.

Trois agents peuvent être créés et coexister, utilisant chacun une CLI différente parmi Goose, OpenCode et Crush. Un message envoyé à chaque agent via WebSocket produit une réponse correcte. Un changement de CLI en cours de conversation préserve l'historique.

La mémoire fonctionne via MCP, le provider est sélectionnable entre Honcho, mcp-memory-service et server-memory, et l'isolation par agent est effective (un agent ne voit pas la mémoire d'un autre).

Le proxy MCP agrège au moins deux serveurs MCP globaux et les expose à toutes les CLI. Le filtrage d'outils par agent fonctionne.

Trois agents peuvent tourner en parallèle, un quatrième est mis en file d'attente. Un agent ne peut pas traiter deux messages simultanés.

Le compacting se déclenche lorsque le seuil configuré est dépassé et produit un résumé qui préserve la continuité de la conversation.

L'interface couvre login, assistant de création du premier agent, tableau de bord, vue d'agent avec chat, édition de configuration globale, édition d'agent, page events, page logs, page sessions.

Le scheduler déclenche des runs d'agents selon la configuration cron.

L'authentification couvre les accès interface et WebSocket. La déconnexion invalide immédiatement la session.

Après un redémarrage du container, tout est restauré : agents, sessions, historiques, configurations.

La CI passe le formatage, le linting, et les tests. La documentation est écrite et couvre les éléments listés dans la section dédiée. Deux à trois agents d'exemple fonctionnels sont fournis. La checklist de release manuelle a été déroulée au moins une fois avec succès.

---

*Fin du PRD v0.2*
