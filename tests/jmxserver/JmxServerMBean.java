// Some random MBean interface
public interface JmxServerMBean {
  public void setThreadCount(int noOfThreads);
  public int getThreadCount();

  public String getSchemaName();
}
