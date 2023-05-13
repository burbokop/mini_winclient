#pragma once

#include <QAbstractListModel>
#include <QQmlParserStatus>
#include <QtQmlIntegration>

class WinClient;
class QTcpServer;

class MiniWinServer : public QAbstractListModel, public QQmlParserStatus
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(quint16 port MEMBER m_port NOTIFY portChanged)
    Q_PROPERTY(bool verbose MEMBER m_verbose)

public:
    explicit MiniWinServer(QObject *parent = nullptr);

signals:
    void portChanged();

    // QAbstractItemModel interface
public:
    virtual int rowCount(const QModelIndex &parent) const override;
    virtual QVariant data(const QModelIndex &index, int role) const override;

    // QQmlParserStatus interface
public:
    void classBegin() override;
    void componentComplete() override;

protected:
    bool listen(quint16 port);

private:
    QTcpServer *m_server = nullptr;
    QVector<WinClient *> m_clients;
    std::uint8_t m_nextId = 0;
    quint16 m_port = 0;
    bool m_verbose = false;
};
