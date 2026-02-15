# -------- Build stage --------
FROM maven:3.9.8-eclipse-temurin-21 AS build
WORKDIR /src

ARG GH_REF=master

RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 --branch "${GH_REF}" https://github.com/graphhopper/graphhopper.git .
RUN mvn -q -DskipTests package

# -------- Runtime stage --------
FROM eclipse-temurin:21-jre

RUN useradd -m -u 10001 graphhopper
WORKDIR /app

COPY --from=build /src/web/target/graphhopper-web-*.jar /app/graphhopper-web.jar
COPY config.yml /app/config.yml

RUN mkdir -p /data/osm /data/graph-cache \
  && chown -R graphhopper:graphhopper /app /data

USER graphhopper
EXPOSE 8989

ENV JAVA_OPTS="-Xms2g -Xmx6g"

ENTRYPOINT ["sh", "-c", "exec java $JAVA_OPTS -jar /app/graphhopper-web.jar server /app/config.yml"]