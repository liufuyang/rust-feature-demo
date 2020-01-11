package com.spotify.apollo.example;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.ObjectWriter;
import com.spotify.apollo.RequestContext;
import com.spotify.apollo.Response;
import com.spotify.apollo.route.AsyncHandler;
import com.spotify.apollo.route.JsonSerializerMiddlewares;
import com.spotify.apollo.route.Route;
import okio.ByteString;

import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Stream;

public class DummyResource {

  private final ObjectWriter objectWriter;
  private final Map<String, Integer> map = new ConcurrentHashMap<>();

  public DummyResource(ObjectMapper objectMapper) {
    map.put("add1", 0);
    map.put("add2", 0);
    map.put("add3", 0);

    this.objectWriter = objectMapper.writer();
  }

  public Stream<Route<AsyncHandler<Response<ByteString>>>> routes() {
    return Stream.of(Route.async("GET", "/add", this::add), Route.async("GET", "/show", this::show),
            Route.async("GET", "/reset", this::reset))
        .map(
            route ->
                route.withMiddleware(
                    JsonSerializerMiddlewares.jsonSerializeResponse(objectWriter)));
  }

  private CompletionStage<Response<Map<String, Integer>>> add(RequestContext requestContext) {
    map.computeIfPresent("add1", (key, value) -> value + 1);
    map.computeIfPresent("add2", (key, value) -> value + 2);
    map.computeIfPresent("add3", (key, value) -> value + 3);
    // map.put("add3", map.get("add3") + 3);

    return CompletableFuture.completedFuture(Response.ok().withPayload(map));
  }

  private CompletionStage<Response<Map<String, Integer>>> show(RequestContext requestContext) {
    return CompletableFuture.completedFuture(Response.ok().withPayload(map));
  }

  private CompletionStage<Response<Map<String, Integer>>> reset(RequestContext requestContext) {
    map.put("add1", 0);
    map.put("add2", 0);
    map.put("add3", 0);
    return CompletableFuture.completedFuture(Response.ok().withPayload(map));
  }
}
