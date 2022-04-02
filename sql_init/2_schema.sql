-- MySQL dump 10.13  Distrib 8.0.28, for Linux (x86_64)
--
-- Host: localhost    Database: fbets
-- ------------------------------------------------------
-- Server version	8.0.28-0ubuntu0.20.04.3

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
-- Table structure for table `CLUB`
--

DROP TABLE IF EXISTS `CLUB`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `CLUB` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(32) NOT NULL,
  `short` varchar(4) NOT NULL,
  `home_stadium_id` int DEFAULT NULL,
  `logo_path` varchar(64) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  PRIMARY KEY (`id`),
  KEY `home_stadium_id` (`home_stadium_id`),
  CONSTRAINT `CLUB_ibfk_1` FOREIGN KEY (`home_stadium_id`) REFERENCES `STADIUM` (`id`) ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `COMPETITION`
--

DROP TABLE IF EXISTS `COMPETITION`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `COMPETITION` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(32) NOT NULL,
  `federation_id` int NOT NULL,
  `logo_path` varchar(64) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `federation_id` (`federation_id`),
  CONSTRAINT `COMPETITION_ibfk_1` FOREIGN KEY (`federation_id`) REFERENCES `FEDERATION` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `EDITION`
--

DROP TABLE IF EXISTS `EDITION`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `EDITION` (
  `id` int NOT NULL AUTO_INCREMENT,
  `competition_id` int NOT NULL,
  `year_begin` int NOT NULL,
  `year_end` int DEFAULT NULL,
  `winner_id` int DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `competition_id` (`competition_id`),
  KEY `winner_id` (`winner_id`),
  CONSTRAINT `EDITION_ibfk_1` FOREIGN KEY (`competition_id`) REFERENCES `COMPETITION` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `EDITION_ibfk_2` FOREIGN KEY (`winner_id`) REFERENCES `CLUB` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `FEDERATION`
--

DROP TABLE IF EXISTS `FEDERATION`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `FEDERATION` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(32) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `FEDERATION_CLUB`
--

DROP TABLE IF EXISTS `FEDERATION_CLUB`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `FEDERATION_CLUB` (
  `id` int NOT NULL AUTO_INCREMENT,
  `federation_id` int NOT NULL,
  `club_id` int NOT NULL,
  PRIMARY KEY (`id`,`federation_id`,`club_id`),
  KEY `club_id` (`club_id`),
  KEY `federation_id` (`federation_id`),
  CONSTRAINT `FEDERATION_CLUB_ibfk_1` FOREIGN KEY (`club_id`) REFERENCES `CLUB` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `FEDERATION_CLUB_ibfk_2` FOREIGN KEY (`federation_id`) REFERENCES `FEDERATION` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `GAME`
--

DROP TABLE IF EXISTS `GAME`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `GAME` (
  `id` int NOT NULL AUTO_INCREMENT,
  `stadium_id` int DEFAULT NULL,
  `home_team_id` int NOT NULL,
  `home_team_odds` float DEFAULT NULL,
  `home_team_score` int unsigned DEFAULT NULL,
  `away_team_id` int NOT NULL,
  `away_team_odds` float DEFAULT NULL,
  `away_team_score` int unsigned DEFAULT NULL,
  `draw_odds` float DEFAULT NULL,
  `edition_id` int NOT NULL,
  `played_on` timestamp NULL DEFAULT NULL,
  `result` enum('W','A','D','C') DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `home_team_id` (`home_team_id`,`away_team_id`,`edition_id`),
  KEY `GAME_ibfk_2` (`away_team_id`),
  KEY `edition_id` (`edition_id`),
  KEY `stadium_id` (`stadium_id`),
  CONSTRAINT `GAME_ibfk_1` FOREIGN KEY (`home_team_id`) REFERENCES `CLUB` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `GAME_ibfk_2` FOREIGN KEY (`away_team_id`) REFERENCES `CLUB` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `GAME_ibfk_3` FOREIGN KEY (`edition_id`) REFERENCES `EDITION` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `GAME_ibfk_4` FOREIGN KEY (`stadium_id`) REFERENCES `STADIUM` (`id`) ON DELETE SET NULL ON UPDATE SET NULL
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `NAVACCESS`
--

DROP TABLE IF EXISTS `NAVACCESS`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `NAVACCESS` (
  `id` int NOT NULL AUTO_INCREMENT,
  `label` varchar(32) NOT NULL,
  `href` varchar(32) NOT NULL,
  `position` int DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `position` (`position`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `ROLE`
--

DROP TABLE IF EXISTS `ROLE`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ROLE` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(16) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `ROLE_NAVACCESS`
--

DROP TABLE IF EXISTS `ROLE_NAVACCESS`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ROLE_NAVACCESS` (
  `id` int NOT NULL AUTO_INCREMENT,
  `role_id` int NOT NULL,
  `navaccess_id` int NOT NULL,
  PRIMARY KEY (`id`,`role_id`,`navaccess_id`),
  UNIQUE KEY `id` (`id`,`role_id`,`navaccess_id`),
  KEY `navaccess_id` (`navaccess_id`),
  KEY `role_id` (`role_id`),
  CONSTRAINT `ROLE_NAVACCESS_ibfk_1` FOREIGN KEY (`navaccess_id`) REFERENCES `NAVACCESS` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `ROLE_NAVACCESS_ibfk_2` FOREIGN KEY (`role_id`) REFERENCES `ROLE` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=21 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `STADIUM`
--

DROP TABLE IF EXISTS `STADIUM`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `STADIUM` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(32) NOT NULL,
  `city` varchar(32) NOT NULL,
  `country` varchar(32) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `USER`
--

DROP TABLE IF EXISTS `USER`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `USER` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT 'ID of user',
  `name` varchar(32) NOT NULL COMMENT 'User name',
  `login` varchar(32) NOT NULL,
  `password` varchar(64) NOT NULL,
  `is_authorized` tinyint(1) NOT NULL DEFAULT '0',
  `role` int NOT NULL DEFAULT '1',
  `joined_on` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `login` (`login`),
  KEY `role` (`role`),
  CONSTRAINT `USER_ibfk_1` FOREIGN KEY (`role`) REFERENCES `ROLE` (`id`) ON DELETE RESTRICT ON UPDATE RESTRICT
) ENGINE=InnoDB AUTO_INCREMENT=26 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `seaql_migrations`
--

DROP TABLE IF EXISTS `seaql_migrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `seaql_migrations` (
  `version` varchar(255) NOT NULL,
  `applied_at` bigint NOT NULL,
  PRIMARY KEY (`version`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2022-03-14 20:27:37
