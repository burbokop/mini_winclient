#pragma once

#include <QQuickPaintedItem>

class WinClient;

class WinClientView : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(WinClient *client READ client WRITE setClient NOTIFY clientChanged)

public:
    WinClientView(QQuickItem *parent = nullptr);

    // QQuickPaintedItem interface
public:
    virtual void paint(QPainter *painter) override;

    WinClient *client() const;
    void setClient(WinClient *newClient);

signals:
    void clientChanged();

private:
    void sync();
    QList<QMetaObject::Connection> m_conns;

    WinClient *m_client = nullptr;
};
