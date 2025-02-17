/*
 * Copyright 2024 The Drasi Authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package io.drasi;

import io.drasi.models.RelationalGraphMapping;
import io.debezium.config.Configuration;
import io.debezium.engine.ChangeEvent;
import io.debezium.engine.DebeziumEngine;
import io.debezium.engine.format.Json;
import io.debezium.engine.spi.OffsetCommitPolicy;
import io.drasi.source.sdk.ChangeMonitor;
import io.drasi.source.sdk.ChangePublisher;
import io.drasi.source.sdk.Reactivator;
import io.drasi.source.sdk.StateStore;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.util.Properties;

public class SqlServerChangeMonitor implements ChangeMonitor {
    private DebeziumEngine<ChangeEvent<String, String>> engine;
    private static final Logger log = LoggerFactory.getLogger(SqlServerChangeMonitor.class);

    public SqlServerChangeMonitor() {
    }

    public void run(ChangePublisher changePublisher, StateStore stateStore) throws Exception {
        var sourceId = Reactivator.SourceId();
        var tableListStr = Reactivator.GetConfigValue("tables");
        var tableList = tableListStr.split(",");

        var mappings = new RelationalGraphMapping();

        Configuration config = Configuration.create()
                .with("connector.class", "io.debezium.connector.sqlserver.SqlServerConnector")
                .with("offset.storage", "io.drasi.OffsetBackingStore")
                .with("offset.flush.interval.ms", 5000)
                .with("name", sourceId)
                .with("topic.prefix", CleanPublicationName(sourceId))
                .with("database.hostname", Reactivator.GetConfigValue("host"))
                .with("database.port", Reactivator.GetConfigValue("port"))
                .with("database.user", Reactivator.GetConfigValue("user", ""))
                .with("database.password", Reactivator.GetConfigValue("password", ""))
                .with("database.encrypt", Reactivator.GetConfigValue("encrypt"))
                .with("database.trustServerCertificate", Reactivator.GetConfigValue("trustServerCertificate", "false"))
                .with("driver.authentication", Reactivator.GetConfigValue("authentication", "NotSpecified"))
                .with("database.names", Reactivator.GetConfigValue("database"))
                .with("tombstones.on.delete", false)
                .with("snapshot.mode", "no_data")
                .with("schema.history.internal", "io.drasi.NoOpSchemaHistory")
                .with("decimal.handling.mode", "double")
                .with("time.precision.mode", "adaptive_time_microseconds")
                .with("converters", "temporalConverter")
                .with("temporalConverter.type", "io.drasi.TemporalConverter")
                .with("table.include.list", tableListStr).build();

        var sr = new SchemaReader(config);
        mappings.nodes = sr.ReadMappingsFromSchema(tableList);

        final Properties props = config.asProperties();

        engine = DebeziumEngine.create(Json.class)
                .using(props)
                .using(OffsetCommitPolicy.always())
                .using(
                        (success, message, error) -> {
                            if (!success && (error != null)) {
                                Reactivator.TerminalError(error);
                            }
                        }
                )
                .notifying(new SqlServerChangeConsumer(mappings, changePublisher)).build();

        engine.run();
    }

    public void close() throws Exception {
        engine.close();
    }

    private static String CleanPublicationName(String name) {
        return name.replace("-", "_");
    }
}
