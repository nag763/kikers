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
INSERT INTO `NAVACCESS` VALUES (1,'Leaderboard','/leaderboard',0),(2,'My bets','/mybets',1),(3,'Administration','/admin',3),(4,'Activation of users','/user/activation',NULL),(5,'Deletion of users','/user/deletion',NULL),(6,'User modification','/user/modification',NULL),(7,'User search','/user/search',NULL),(8,'Games','/games',2),(10,'See profile','/profile/edit',NULL),(11,'Favorite user\'s leagues','/profile/leagues',NULL);
/*!40000 ALTER TABLE `NAVACCESS` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping data for table `ROLE_NAVACCESS`
--

LOCK TABLES `ROLE_NAVACCESS` WRITE;
/*!40000 ALTER TABLE `ROLE_NAVACCESS` DISABLE KEYS */;
INSERT INTO `ROLE_NAVACCESS` VALUES (1,1,1),(3,2,1),(5,3,1),(4,2,2),(6,3,2),(7,3,3),(8,3,4),(9,3,5),(10,3,6),(11,3,7),(12,1,8),(13,2,8),(14,3,8),(18,1,10),(19,2,10),(20,3,10),(21,1,11),(22,2,11),(23,3,11);
/*!40000 ALTER TABLE `ROLE_NAVACCESS` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2022-04-03 19:46:34
