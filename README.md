# Kafka Consent to Onkostar

Diese Anwendung nimmt IDAT Kafka Consent Records entgegen und versendet diese an Onkostar.

## Konfiguration

Beim Start der Anwendung können Parameter angegeben werden.

```
Usage: kafka-consent-to-onkostar [OPTIONS] <ONKOSTAR_URI>

Arguments:
  <ONKOSTAR_URI>  

Options:
      --bootstrap-servers <BOOTSTRAP_SERVERS>
          Kafka Bootstrap Server [env: KAFKA_BOOTSTRAP_SERVERS=] [default: kafka:9094]
      --topic <TOPIC>
          Kafka Topic [env: KAFKA_TOPIC=] [default: consent-json-idat]
      --group-id <GROUP_ID>
          Kafka Group ID [env: KAFKA_GROUP_ID=] [default: kafka-consent-to-onkostar]
      --ssl-ca-file <SSL_CA_FILE>
          CA file for SSL connection to Kafka [env: KAFKA_SSL_CA_FILE=]
      --ssl-cert-file <SSL_CERT_FILE>
          Certificate file for SSL connection to Kafka [env: KAFKA_SSL_CERT_FILE=]
      --ssl-key-file <SSL_KEY_FILE>
          Key file for SSL connection to Kafka [env: KAFKA_SSL_KEY_FILE=]
      --ssl-key-password <SSL_KEY_PASSWORD>
          The SSL key password [env: KAFKA_SSL_KEY_PASSWORD=]
      --onkostar-username <ONKOSTAR_USERNAME>
          Onkostar Username [env: ONKOSTAR_USERNAME=]
      --onkostar-password <ONKOSTAR_PASSWORD>
          Onkostar Password [env: ONKOSTAR_PASSWORD=]
```

Die Anwendung lässt sich auch mit Umgebungsvariablen konfigurieren.

* `KAFKA_BOOTSTRAP_SERVERS`: Zu verwendende Kafka-Bootstrap-Server als kommagetrennte Liste
* `KAFKA_TOPIC`: Zu verwendendes Topic zum Warten auf neue Anfragen. Standardwert: `consent-json-idat`
* `KAFKA_GROUP_ID`: Die Kafka Group ID. Standardwert: `kafka-consent-to-onkostar`
* `ONKOSTAR_URI`: URI für Onkostar API Requests (z.B. http://localhost:8080/onkostar)

Optionale Umgebungsvariablen für Onkostar.
Wenn angegeben, werden der Benutzername und das Password benutzt.

* `ONKOSTAR_USERNAME`: Benutzername für Onkostar (wenn erforderlich)
* `ONKOSTAR_PASSWORD`: Passwort für Onkostar (wenn erforderlich)

Optionale Umgebungsvariablen für Kafka - wenn angegeben wird eine SSL-Verbindung zu Kafka aufgebaut.

* `KAFKA_SSL_CA_FILE`: CA für SSL-Verbindungen
* `KAFKA_SSL_CERT_FILE`: SSL Certificate Datei
* `KAFKA_SSL_KEY_FILE`: SSL Key Datei
* `KAFKA_SSL_KEY_PASSWORD`: SSL Key Passwort (wenn benötigt)

## Lizenz

AGPL-3.0