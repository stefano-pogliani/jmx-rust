// Some random MBean implementation
public class JmxServer implements JmxServerMBean {
  private int threadCount;
  private String schemaName;

  public JmxServer(int numThreads, String schema) {
    this.threadCount = numThreads;
    this.schemaName = schema;
  }

  @Override
  public void setThreadCount(int noOfThreads) {
    this.threadCount = noOfThreads;
  }
  @Override
  public int getThreadCount() {
    return this.threadCount;
  }

  @Override
  public String getSchemaName() {
    return this.schemaName;
  }
}
