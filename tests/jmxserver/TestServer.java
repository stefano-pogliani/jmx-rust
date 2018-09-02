import javax.management.*;
import java.lang.management.*;

public class TestServer {
  private MBeanServer mbs = null;

  public TestServer() {
    mbs = ManagementFactory.getPlatformMBeanServer();

    // Unique identification of MBeans
    JmxServer serverBean = new JmxServer(16, "test");
    ObjectName serverName = null;

    try {
      // Uniquely identify the MBeans and register them with the platform MBeanServer
      serverName = new ObjectName("FOO:name=ServerBean");
      mbs.registerMBean(serverBean, serverName);
    } catch(Exception e) {
      e.printStackTrace();
    }
  }

  private static void waitAWhile() {
    // Waits for 60 seconds before exiting.
    // If a test needs more then 60 seconds it may need to be broken into multiple tests.
    // This is done so that failed tests don't leave loose processes forever.
    try {
      Thread.sleep(60000);
    } catch(Exception e) {
      e.printStackTrace();
    }
  }

  public static void main(String argv[]) {
    TestServer server = new TestServer();
    System.out.println("TestServer is running...");
    TestServer.waitAWhile();
  }
}
