-- MySQL dump 10.13  Distrib 8.0.29, for Linux (x86_64)
--
-- Host: localhost    Database: fbets
-- ------------------------------------------------------
-- Server version	8.0.29-0ubuntu0.20.04.3

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Dumping data for table `ROLE`
--

LOCK TABLES `ROLE` WRITE;
/*!40000 ALTER TABLE `ROLE` DISABLE KEYS */;
INSERT INTO `ROLE` VALUES (1,'user'),(2,'manager'),(3,'admin');
/*!40000 ALTER TABLE `ROLE` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `NAVACCESS`
--

LOCK TABLES `NAVACCESS` WRITE;
/*!40000 ALTER TABLE `NAVACCESS` DISABLE KEYS */;
INSERT INTO `NAVACCESS` VALUES (1,'M00010_LEADERBOARD','M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z','/leaderboard',1),(2,'M00010_BETS','M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z','/mybets',NULL),(3,'M00010_ADMIN','M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z','/admin/users',3),(4,'Activation of users',NULL,'/user/activation',NULL),(5,'Deletion of users',NULL,'/user/deletion',NULL),(6,'User modification',NULL,'/user/modification',NULL),(7,'User search',NULL,'/user/search',NULL),(8,'M00010_GAMES','M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z','/games',2),(10,'See profile',NULL,'/profile/edit',NULL),(11,'Favorite user\'s leagues',NULL,'/profile/leagues',NULL),(12,'Favorite clubs',NULL,'/profile/clubs',NULL),(13,'Games status update',NULL,'/games/update/status',NULL),(14,'Clubs search',NULL,'/clubs/search',NULL),(15,'Bookmakers',NULL,'/admin/bookmakers',NULL),(16,'Bets on game',NULL,'/games/bet',NULL),(17,'Season choser',NULL,'/admin/seasons',NULL),(18,'Add new season',NULL,'/admin/season/add',NULL),(19,'Set main season',NULL,'/admin/season/set_main',NULL),(20,'Close the season',NULL,'/admin/season/close',NULL),(21,'M00010_NEWS','\r\nM19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z','/',0);
/*!40000 ALTER TABLE `NAVACCESS` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `ROLE_NAVACCESS`
--

LOCK TABLES `ROLE_NAVACCESS` WRITE;
/*!40000 ALTER TABLE `ROLE_NAVACCESS` DISABLE KEYS */;
INSERT INTO `ROLE_NAVACCESS` VALUES (1,1,1),(3,2,1),(5,3,1),(4,2,2),(6,3,2),(7,3,3),(8,3,4),(9,3,5),(10,3,6),(11,3,7),(12,1,8),(13,2,8),(14,3,8),(18,1,10),(19,2,10),(20,3,10),(21,1,11),(22,2,11),(23,3,11),(24,3,12),(25,2,12),(26,1,12),(27,2,13),(28,3,13),(29,1,14),(30,2,14),(31,3,14),(32,3,15),(33,1,16),(34,2,16),(35,3,16),(37,3,17),(38,3,18),(39,3,19),(40,3,20),(42,1,21),(43,2,21),(44,3,21);
/*!40000 ALTER TABLE `ROLE_NAVACCESS` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `LABEL`
--

LOCK TABLES `LABEL` WRITE;
/*!40000 ALTER TABLE `LABEL` DISABLE KEYS */;
INSERT INTO `LABEL` VALUES (1,'HOME_WELCOME_BACK','Welcome back !'),(2,'HOME_PARAGRAPH','This is some random text right here'),(3,'FORM_LOGIN','Login'),(4,'FORM_PASSWORD','Password'),(5,'FORM_NAME','Name'),(6,'FORM_LANGUAGE','Language'),(7,'FORM_ROLE','Role'),(8,'FORM_ACCESS','Can access'),(9,'FORM_SAVE','Save'),(10,'FORM_DELETE','Delete'),(11,'FORM_EDIT','Edit'),(12,'FORM_ACTIVATION','Activation'),(13,'FORM_ACTIVATE','Activate'),(14,'FORM_DEACTIVATE','Deactivate'),(15,'FORM_DELETION','Deletion'),(16,'COMMON_NEXT','Next'),(17,'COMMON_PREVIOUS','Previous'),(18,'COMMON_SEARCH_LOGIN','Search login'),(19,'SIDE_PERSONNAL_INFOS','Your informations'),(20,'SIDE_FAV_LEAGUES','Your favorite leagues'),(21,'M2001_SIDE_YOUR_INFOS','Your informations'),(22,'M2001_SIDE_YOUR_LEAGUES','Your favorite leagues'),(23,'M2001_SIDE_YOUR_CLUBS','Your favorite clubs'),(24,'M2003_SEARCH_RESULTS','Search results'),(25,'M2003_SEARCH_RESULTS_DESC','Find below the results of the search you just made, click on go back if you want to display your favorite leagues again'),(26,'M2003_SEARCH_RESULTS_NONE','No results are matching your search criteria, please try again with another wording.'),(27,'M2003_TITLE','Your favorite leagues'),(28,'M2003_TITLE_DESC','Find below your favorite leagues'),(29,'M2003_NO_FAV','You don\'t have any favorite league so far, add some by using the country picker first.'),(30,'M2003_SEARCH_ACTION','Search a league'),(31,'M2003_SEARCH_ACTION_DESC','If you want to add or remove a particular league from your profile, you can use the search bar below.'),(32,'M2004_SEARCH_RESULTS','Search results'),(33,'M2004_NO_MATCH','No results are matching your research'),(34,'M2004_YOUR_FAV','Your favorite clubs'),(35,'M2004_YOUR_FAV_DESC','Find below your favorite clbus'),(36,'M2004_NO_FAV','You don\'t have any club so far that has been added to your profile, do a search and mark the clubs you like as favorites.'),(37,'M2004_SEARCH_ACTION','Search for a club'),(38,'M2004_SEARCH_ACTION_DESC','Type the club name you want to do a search for on the search bar below'),(47,'M2004_TITLE','Your favorite clubs'),(49,'M2002_TITLE','Your informations'),(50,'M10010_AET','after extra time'),(51,'M10010_ON_PENS','on pens'),(52,'M10010_STARTS_IN','Starts in'),(53,'COMMON_MINUTES','minutes'),(54,'COMMON_HOURS','hours'),(55,'COMMON_AND','and'),(56,'M10010_RESULT_UNKNOWN','Result unknown for this game'),(57,'M10001_TODAY_NO','No games available for today'),(58,'M10001_YESTERDAY_NO','Yesterday games aren\'t available'),(59,'M10001_TOMOROW_NO','Tomorow games aren\'t available'),(60,'M10002_NO_GAMES','No games available for that day, or you might have not added games to your favorites yet if you aren\'t in see all games mode.'),(61,'M10002_ADD_MORE','To see more games, edit your settings and add clubs or leagues to your favorites'),(62,'M10011_FAVORITE','Favorites only'),(63,'M10011_ALL','All games'),(64,'M10001_TOMOROW_TITLE','Tomorow games'),(65,'M10001_YESTERDAY_TITLE','Yesterday games'),(66,'M10001_TODAY_TITLE','Today games'),(67,'M10001_TITLE','Games'),(68,'M10001_GAME_OF_DAY','Games of the day'),(69,'M00010_LEADERBOARD','Leaderboard'),(70,'M00010_BETS','My bets'),(71,'M00010_ADMIN','Administration'),(72,'M00010_GAMES','Games'),(73,'M30001_TITLE','User management'),(74,'COMMON_GO_BACK','Go back'),(75,'M10010_SEE_MORE','see more'),(76,'M10010_LAST_UPDATED','Last updated on'),(77,'M2003_SEARCH_BAR_LEAGUE','Search a league'),(78,'M2004_SEARCH_BAR_CLUB','Search a club'),(79,'M30002_TITLE','Bookmakers'),(80,'M30002_PICK_A_BM','Use this screen to pick the bookmaker that should be used for the odds :'),(81,'HOME_LATEST_NEWS','Latest news'),(82,'M10011_DEFAULT','Favorites and bets'),(83,'M10011_BETS','Bets only'),(84,'M10011_POTENTIAL_BETS','Potential bets only'),(85,'M30003_TITLE','Seasons'),(86,'M30003_PICK_A_SZN','Pick the season you want to modify.'),(87,'FORM_CLOSE','Close'),(88,'M30003_SET_AS_MAIN','Set as current season'),(89,'M30003_INFO','The main season is prefixed with a star thus be aware that you have to set another season as current if you want to close the current one.'),(90,'M30003_ADD_LEAGUE_PH','Type the new season name right here'),(91,'M30003_ADD_SZN_INFO','You can add a new season by typing its name right below'),(92,'M30003_HEADER_1','Add a new season'),(93,'M30003_HEADER_2','Change a season\'s status'),(95,'M4001_TITLE','Leaderboard'),(96,'M40001_TITLE','Leaderboard'),(97,'COMMON_ALL','All'),(98,'M40001_ALL_SEASONS','All seasons'),(99,'M40001_ALL_TIME_RESULT','All time result'),(100,'M40001_NO_RESULT','There is no result for this season'),(101,'M40001_RANKING','Ranking'),(102,'M4001_POINTS','Points'),(103,'M40001_NUMBER_OF_BETS','Number of bets made'),(104,'M40001_PPB','Points per bet'),(105,'M40001_SEASON','Season'),(106,'M40001_PLAYER','Player'),(107,'M40001_POINTS','Points'),(108,'M00010_NEWS','News'),(109,'HOME_TOP3','Top 3');
/*!40000 ALTER TABLE `LABEL` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `LANGUAGE`
--

LOCK TABLES `LANGUAGE` WRITE;
/*!40000 ALTER TABLE `LANGUAGE` DISABLE KEYS */;
INSERT INTO `LANGUAGE` VALUES (1,'english','en'),(2,'french','fr');
/*!40000 ALTER TABLE `LANGUAGE` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `LOCALE`
--

LOCK TABLES `LOCALE` WRITE;
/*!40000 ALTER TABLE `LOCALE` DISABLE KEYS */;
INSERT INTO `LOCALE` VALUES (1,1,'en-US','English (US)'),(2,2,'fr-FR','Français'),(3,1,'en-GB','English (UK)');
/*!40000 ALTER TABLE `LOCALE` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `TRANSLATION`
--

LOCK TABLES `TRANSLATION` WRITE;
/*!40000 ALTER TABLE `TRANSLATION` DISABLE KEYS */;
INSERT INTO `TRANSLATION` VALUES (3,1,2,'Bon retour'),(4,2,2,'Bon retour parmis nous'),(5,3,2,'Identifiant'),(6,4,2,'Mot de passe'),(7,5,2,'Nom'),(8,6,2,'Langage'),(9,7,2,'Role'),(10,8,2,'Peut accèder à l’application'),(11,9,2,'Sauvegarder'),(12,10,2,'Supprimer'),(13,11,2,'Editer'),(14,12,2,'Activation'),(15,13,2,'Activer'),(16,14,2,'Désactivation'),(17,15,2,'Suppression'),(18,16,2,'Suivant'),(19,17,2,'Précèdent'),(20,18,2,'Rechercher un nom d’utilisateur'),(21,19,2,'Vos informations'),(22,20,2,'Vos ligues favorites'),(23,21,2,'Vos informations'),(24,22,2,'Vos ligues favorites'),(25,23,2,'Vos clubs favoris'),(26,24,2,'Résultat de la recherche'),(27,25,2,'Trouvez ci-dessous les résultats de la recherche, cliquez sur retour si vous souhaitez retournez à l’écran précèdent'),(28,26,2,'Aucun résultat n’a été trouvé pour votre recherche, réessayez avec une autre formulation'),(29,27,2,'Vos ligues favorites'),(30,28,2,'Trouvez ci-dessous vos ligues favorites'),(31,29,2,'Vous n’avez pas de ligues dans vos favoris pour le moment, vous pouvez en ajoutez en utilisant la barre de recherche à droite'),(32,30,2,'Rechercher une ligue'),(33,31,2,'Si vous souhaitez ajouter ou supprimer une ligue de vos favoris, utilisez la barre de recherche ci-dessous'),(34,32,2,'Résultat de la recherche'),(35,33,2,'Aucun résultat n’a été trouvé pour votre recherche, réessayez avec une autre formulation'),(36,34,2,'Vos clubs favoris'),(37,35,2,'Trouvez ci-dessous vos clubs favoris'),(38,36,2,'Vous n’avez pas de clubs dans vos favoris pour le moment, vous pouvez en ajoutez en utilisant la barre de recherche à droite'),(39,37,2,'Rechercher un club'),(40,38,2,'Tapez le nom du club que vous souhaitez rechercher'),(41,47,2,'Vos clubs favoris'),(42,49,2,'Vos informations'),(43,50,2,'après temps additionnel'),(44,51,2,'sur pénalties'),(45,52,2,'Commence dans'),(46,53,2,'minutes'),(47,54,2,'heures'),(48,55,2,'et'),(49,56,2,'Résultat non connu pour ce match'),(50,57,2,'Pas de matchs disponibles pour ce jour'),(51,58,2,'Les matchs d’hier ne sont pas disponibles'),(52,59,2,'Les matchs de demain ne sont pas disponibles'),(53,60,2,'Pas de matchs trouvés pour ce jour selon les critères que vous avez choisi'),(54,61,2,'Ajoutez des clubs ou des ligues dans vos favoris pour potentiellement voir plus de résultats de recherche'),(55,62,2,'Seulement les favoris'),(56,63,2,'Tous les matchs'),(57,64,2,'Matchs de demain'),(58,65,2,'Matchs d’hier'),(59,66,2,'Matchs d’aujourd’hui'),(60,67,2,'Matchs'),(61,68,2,'Matchs du jour'),(62,69,2,'Tableau des scores'),(63,70,2,'Mes paris'),(64,71,2,'Administration'),(65,72,2,'Matchs'),(66,73,2,'Gestion des utilisateurs'),(67,74,2,'Retour en arrière'),(68,75,2,'voir plus'),(69,76,2,'Dernière mise à jour le'),(70,77,2,'Rechercher une ligue'),(71,78,2,'Chercher un club'),(72,80,2,'Utilisez le champ ci-dessous pour choisir un bookmaker pour l\'application :'),(73,81,2,'Sur le fil'),(74,83,2,'Seulement les paris'),(75,82,2,'Favoris et paris'),(76,84,2,'Seulement les paris potentiels'),(77,85,2,'Saisons'),(78,87,2,'Clôturer '),(79,88,2,'Définir comme la saison courante'),(80,89,2,'La saison courante a pour préfixe une étoile, veillez à définir une nouvelle saison courante si vous souhaitez la clôturer'),(81,90,2,'Tapez le nom de la nouvelle saison ici'),(82,91,2,'Vous pouvez ajouter une nouvelle saison en tapant son nom ci-dessous'),(83,92,2,'Ajouter une nouvelle saison'),(92,93,2,'Changer le statut d\'une saison'),(93,86,2,'Choisissez la saison que vous souhaitez modifier'),(94,97,2,'Tous'),(95,98,2,'Toute saison confondue'),(96,99,2,'Résultats pour toutes les saisons confondues'),(97,105,2,'Saison'),(98,106,2,'Joueur'),(99,108,2,'Actus');
/*!40000 ALTER TABLE `TRANSLATION` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `RESULT`
--

LOCK TABLES `RESULT` WRITE;
/*!40000 ALTER TABLE `RESULT` DISABLE KEYS */;
INSERT INTO `RESULT` VALUES (1,'Win'),(2,'Draw'),(3,'Loss');
/*!40000 ALTER TABLE `RESULT` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2022-07-18 20:53:19
