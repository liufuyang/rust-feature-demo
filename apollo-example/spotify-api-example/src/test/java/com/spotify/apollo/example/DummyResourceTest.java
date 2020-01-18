package com.spotify.apollo.example;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import org.junit.Test;

public class DummyResourceTest {


  private int parallelSumTo10000(Map<String, Integer> map) throws InterruptedException {

    map.put("test", 0);
    ExecutorService executorService = Executors.newFixedThreadPool(10);

    for (int j = 0; j < 100; j++) {
      executorService.execute(
          () -> {
            for (int k = 0; k < 100; k++) {
              map.put("test", map.get("test") + 1);
            }
          });
    }
    executorService.shutdown();
    executorService.awaitTermination(5, TimeUnit.SECONDS);

    return map.get("test");
  }

  @Test
  public void givenHashMap_whenSumParallel_thenError() throws Exception {
    Map<String, Integer> map = new HashMap<>();
    int sum = parallelSumTo10000(map);
    assertEquals(10000, sum);
  }

  @Test
  public void givenConcurrentMap_whenSumParallel_thenCorrect() throws Exception {
    Map<String, Integer> map = new ConcurrentHashMap<>();
    int sum = parallelSumTo10000(map);
    assertEquals(10000, sum);
  }

}



// map.computeIfPresent("test", (key, value) -> value + 1);